use anyhow::Context;
use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::startup::AppState;

use super::subscriptions::SubscribeError;

#[derive(Debug, Deserialize)]
pub struct ConfirmQuery {
    subscription_token: String,
}

pub async fn confirm_subscriber(
    State(state): State<AppState>,
    Query(params): Query<ConfirmQuery>,
) -> Result<impl IntoResponse, SubscribeError> {
    let mut tx = state
        .pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    let id = get_subscriber_id_from_token(&params.subscription_token, &mut tx)
        .await
        .context("Failed to get subscriber id")?;

    let _ = update_to_confirmed(&mut tx, id)
        .await
        .context("Failed to update subscriber status")?;
    tx.commit().await.context("Failed to commit transaction")?;
    Ok(StatusCode::OK.into_response())
}

async fn get_subscriber_id_from_token<'a>(
    token: &'a str,
    trans: &mut Transaction<'_, Postgres>,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = sqlx::query!(
        r#"SELECT subscriber_id FROM subscriptions_tokens WHERE subscription_tokens = $1"#,
        &token
    )
    .fetch_optional(&mut **trans)
    .await
    .map_err(|e| {
        tracing::error!("Failed trying to get subscriber id: {:?}", e);
        e
    })?;
    match subscriber_id {
        Some(id) => Ok(id.subscriber_id),
        None => Err(sqlx::Error::RowNotFound),
    }
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, trans))]
pub async fn update_to_confirmed(
    trans: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id,
    )
    .execute(&mut **trans)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
