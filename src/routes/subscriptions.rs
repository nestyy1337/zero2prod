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

pub async fn subscribe(
    State(pool): State<PgPool>,
    Form(sub_form): Form<SubscribeForm>,
) -> Response {
    let query_span = tracing::info_span!("Saving new subscriber details in the database");
    let query = sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        sub_form.email,
        sub_form.name,
        Utc::now()
    )
    .execute(&pool)
    .instrument(query_span)
    .await;

    match &query {
        Ok(_) => {
            tracing::info!(
                "Successfully saved new user with name: {}, email: {}",
                &sub_form.name,
                &sub_form.email
            );
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
                        &sub_form
                    );
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
            err => {
                tracing::error!(
                    "Error: {:?} trying to insert new subscriber: {:?}",
                    err,
                    &sub_form
                );
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        },
    }
}
