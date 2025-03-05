use anyhow::Context;
use axum::{
    extract::{self, State},
    response::{IntoResponse, Response},
};
use hyper::StatusCode;
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    domain::{SubscriberEmail, SubscriberName},
    email_client::EmailBody,
    idempotency::persistance::{
        generate_idempotency_key, get_existing_job, update_job_status, EmailStatus,
    },
    startup::AppState,
};

#[derive(Debug)]
struct ConfirmedSubscriber {
    email: String,
    name: String,
    id: Uuid,
}

#[derive(Debug, Error)]
pub enum PublishError {
    #[error("test err")]
    TestErr,
    #[error("generic error")]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for PublishError {
    fn into_response(self) -> Response {
        let response = match self {
            PublishError::TestErr => (StatusCode::BAD_REQUEST, "test err".to_string()),
            PublishError::UnexpectedError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
        response.into_response()
    }
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let query = sqlx::query_as!(
        ConfirmedSubscriber,
        r#"
        SELECT email, name, id
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?;

    let query = query
        .iter()
        // shit is cursed but hey, it is what it is
        // cant convert ConfirmedSubscriber into SubscriberEmail
        // since then 'a would be tied to lifetime of this fn
        // which is too short
        .map(|r| match SubscriberEmail::parse(&r.email) {
            Ok(email) => Ok(ConfirmedSubscriber {
                email: email.as_ref().to_string(),
                name: SubscriberName::parse(&r.name)
                    .unwrap_or_default()
                    .as_ref()
                    .to_string(),
                id: r.id,
            }),
            Err(_) => Err(anyhow::anyhow!("Invalid email")),
        })
        .collect::<Vec<Result<ConfirmedSubscriber, anyhow::Error>>>();

    Ok(query)
}

pub async fn publish_newsletter(
    State(state): State<AppState>,
    extract::Json(payload): extract::Json<EmailBody>,
) -> Result<Response, PublishError> {
    let recipients = get_confirmed_subscribers(&state.pool)
        .await
        .context("Failed to get confirmed subscribers")?;

    let mut count = 0;
    let mut successes = vec![];
    let mut errors = vec![];

    for recipient in &recipients {
        match &recipient {
            Ok(subscriber) => {
                let key = generate_idempotency_key(&subscriber.name, &payload.message);
                let job = get_existing_job(&state.pool, &key)
                    .await
                    .context("Failed to get existing job")?;

                match job {
                    Some(job) if matches!(job.status, EmailStatus::Sent) => {
                        tracing::info!("Skipping subscriber, email already sent: {:?}", subscriber);
                        continue;
                    }
                    Some(mut job) if matches!(job.status, EmailStatus::Pending) => {
                        job.attempts += 1;
                        job.updated_at = chrono::Utc::now();
                        sqlx::query!(
                            r#"
                            UPDATE idempotency
                            SET attempts = $1, updated_at = $2
                            WHERE idempotency_key = $3
                            "#,
                            job.attempts,
                            job.updated_at,
                            key
                        )
                        .execute(&state.pool)
                        .await
                        .context("Failed to update job")?;

                        return Ok(StatusCode::OK.into_response());
                    }
                    Some(_) if matches!(job.unwrap().status, EmailStatus::Failed) => {
                        sqlx::query!(
                            r#"
                            UPDATE idempotency
                            SET status = $1
                            WHERE idempotency_key = $2
                            "#,
                            EmailStatus::Pending as EmailStatus,
                            key
                        )
                        .execute(&state.pool)
                        .await
                        .context("Failed to update job")?;
                    }
                    None => {
                        let _ = sqlx::query!(
                            r#"
                            INSERT INTO idempotency (user_id, idempotency_key, status, attempts, created_at, updated_at)
                            VALUES ($1, $2, $3, $4, $5, $6)
                            "#,
                            subscriber.id,
                            key,
                            EmailStatus::Pending as EmailStatus,
                            0,
                            chrono::Utc::now(),
                            chrono::Utc::now()
                        )
                        .execute(&state.pool)
                        .await;
                    }
                    _ => {}
                }

                match state
                    .email_client
                    .send_newsletter(
                        &subscriber.email,
                        &payload.title,
                        &subscriber.name,
                        "newsletter",
                        &key,
                        &state,
                    )
                    .await
                    .with_context(|| format!("Failed to send email to recipient: {:?}", recipient))
                {
                    Ok(success) => {
                        // Update job status to Sent
                        if let Err(e) =
                            update_job_status(&state.pool, &key, EmailStatus::Sent).await
                        {
                            tracing::error!("Failed to update job status: {:?}", e);
                        }
                        successes.push(success);
                    }
                    Err(e) => {
                        // Update job status to Failed
                        if let Err(update_err) =
                            update_job_status(&state.pool, &key, EmailStatus::Failed).await
                        {
                            tracing::error!("Failed to update job status: {:?}", update_err);
                        }
                        errors.push(e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Skipping subscriber, their stored details are invalid: {:?}",
                    e
                );
            }
        }
        count += 1;
    }

    let response_message = format!(
        "Sent updates to {} subscribers, errored with: {}",
        count,
        errors.len()
    );

    Ok((StatusCode::OK, response_message).into_response())
}
