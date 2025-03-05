use anyhow::Context;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Form,
};
use hyper::StatusCode;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Deserialize;
use sqlx::{PgPool, Postgres, Transaction};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    domain::{ParseError, Subscriber},
    email_client::EmailClient,
    startup::AppState,
};

#[derive(Error, Debug)]
pub enum SubscribeError {
    #[error("validation failed for string `{0}`")]
    ValidationError(#[from] ParseError),
    #[error("unexpected error: `{0}`")]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for SubscribeError {
    fn into_response(self) -> Response {
        let response = match self {
            SubscribeError::ValidationError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            SubscribeError::UnexpectedError(e) => match e.downcast_ref::<sqlx::Error>() {
                Some(sqlx::Error::PoolTimedOut) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                ),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            },
        };
        response.into_response()
    }
}

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
) -> Result<impl IntoResponse, SubscribeError> {
    let mut tx = state
        .pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    let subscriber = Subscriber::new(&sub_form.name, &sub_form.email)?;

    let uuid = subscriber
        .try_insert(&mut tx)
        .await
        .context("Failed to insert new subscriber in the database.")?;

    let subsciption_token = generate_subscription_token();

    store_token(&uuid, &subsciption_token, &mut tx)
        .await
        .context("Failed to store the confirmation token for a new subscriber.")?;

    send_confirmation(&state.email_client, &subscriber, &subsciption_token)
        .await
        .context("Failed to send a confirmation email.")?;

    tx.commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber.")?;

    Ok(StatusCode::OK.into_response())
}

async fn store_token(
    subscriber_id: &Uuid,
    token: &str,
    transaction: &mut Transaction<'_, Postgres>,
) -> Result<(), sqlx::Error> {
    let _query = sqlx::query!(
        r#"INSERT INTO subscriptions_tokens (subscription_tokens, subscriber_id)
        VALUES ($1, $2)"#,
        token,
        subscriber_id
    )
    .execute(&mut **transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

async fn send_confirmation<'a>(
    client: &EmailClient,
    subscriber: &Subscriber<'a>,
    token: &str,
) -> Result<(), SubscribeError> {
    let confirmation_link = format!(
        "http://localhost:8000/subscribe/confirm?subscription_token={}",
        &token
    );
    println!("Confirmation link: {}", confirmation_link);
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
