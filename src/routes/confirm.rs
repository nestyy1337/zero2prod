use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{domain::SubscriberEmail, startup::AppState};

#[derive(Debug, Deserialize)]
pub struct ConfirmQuery {
    subscription_token: String,
}

pub async fn confirm_subscriber(
    State(state): State<AppState>,
    Query(params): Query<ConfirmQuery>,
) -> Response {
    let id = match get_subscriber_id_from_token(&params.subscription_token, &state.pool).await {
        Ok(id) => id,
        Err(e) => return StatusCode::BAD_REQUEST.into_response(),
    };

    match update_to_confirmed(&state.pool, id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => StatusCode::BAD_REQUEST.into_response(),
    }
}

async fn get_subscriber_id_from_token<'a>(
    token: &'a str,
    pool: &'a PgPool,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = sqlx::query!(
        r#"SELECT subscriber_id FROM subscriptions_tokens WHERE subscription_tokens = $1"#,
        &token
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::info!("Failed trying to get subscriber id: {:?}", e);
        e
    })?;
    match subscriber_id {
        Some(id) => Ok(id.subscriber_id),
        None => Err(sqlx::Error::RowNotFound),
    }
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, pool))]
pub async fn update_to_confirmed(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
