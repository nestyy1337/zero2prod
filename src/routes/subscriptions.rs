use axum::{
    Form,
    extract::State,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct SubscribeForm {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding new subscriber",
    skip(sub_form, pool),
    fields(
    subscriber_email = %sub_form.email,
    subscriber_name = %sub_form.name,
))]
pub async fn subscribe(
    State(pool): State<PgPool>,
    Form(sub_form): Form<SubscribeForm>,
) -> Response {
    match &insert_subscriber(&pool, &sub_form).await {
        Ok(_) => {
            return StatusCode::CREATED.into_response();
        }
        Err(e) => match e {
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
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
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
    }
}

#[tracing::instrument(
    name = "Inserting a new subscriber",
    skip(form, pool),
    fields(
    subscriber_email = %form.email,
    subscriber_name = %form.name,
))]
async fn insert_subscriber(pool: &PgPool, form: &SubscribeForm) -> Result<(), sqlx::Error> {
    let query_span = tracing::info_span!("Saving new subscriber details in the database");
    let _query = sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool)
    .instrument(query_span)
    .await?;
    Ok(())
}
