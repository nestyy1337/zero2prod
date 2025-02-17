use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Form,
};
use hyper::StatusCode;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Deserialize;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

use crate::{domain::Subscriber, email_client::EmailClient, startup::AppState, AppError};

#[derive(Deserialize, Debug)]
pub struct SubscribeForm {
    pub email: String,
    pub name: String,
}

#[tracing::instrument(
    name = "Adding new subscriber",
    skip(sub_form, state),
    fields(
    subscriber_email = %sub_form.email,
    subscriber_name = %sub_form.name,
))]
pub async fn subscribe(
    State(state): State<AppState>,
    Form(sub_form): Form<SubscribeForm>,
) -> Response {
    let subscriber = match Subscriber::new(&sub_form.name, &sub_form.email) {
        Ok(sub) => sub,
        Err(parse_err) => {
            return Response::from(parse_err);
        }
    };

    let uuid = match subscriber.try_insert(&state.pool).await {
        Ok(new_subscriber_uuid) => new_subscriber_uuid,

        Err(e) => match &e {
            sqlx::Error::Database(db_err) => {
                if db_err.is_unique_violation() {
                    tracing::warn!("Conflicting email: {} already in db", &sub_form.email);
                    return StatusCode::CONFLICT.into_response();
                } else {
                    tracing::error!(
                        "Error: {:?} trying to insert new subscriber: {:?}",
                        e,
                        &sub_form.email
                    );
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }
            err => {
                tracing::error!(
                    "Error: {:?} trying to insert new subscriber: {:?}",
                    err,
                    &sub_form.email
                );
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        },
    };
    let subsciption_token = generate_subscription_token();

    if store_token(&uuid, &subsciption_token, &state.pool)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    match send_confirmation(&state.email_client, &subscriber, &subsciption_token).await {
        Ok(_) => return StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };
    StatusCode::CREATED.into_response()
}

async fn store_token(subscriber_id: &Uuid, token: &str, pool: &PgPool) -> Result<(), sqlx::Error> {
    let _query = sqlx::query!(
        r#"INSERT INTO subscriptions_tokens (subscription_tokens, subscriber_id)
        VALUES ($1, $2)"#,
        token,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    });
    Ok(())
}

async fn send_confirmation<'a>(
    client: &EmailClient,
    subscriber: &Subscriber<'a>,
    token: &str,
) -> Result<(), AppError> {
    let confirmation_link = format!(
        "http://localhost:8000/subscribe/confirm?subscription_token={}",
        &token
    );
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    let subject = "Welcome!";

    let _ = client
        .send_email_example(&subscriber.email, &subject, &html_body, &plain_body)
        .await;
    Ok(())
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
