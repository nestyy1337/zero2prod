use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Form,
};
use hyper::StatusCode;
use serde::Deserialize;

use crate::{domain::Subscriber, startup::AppState};

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

    match subscriber.try_insert(&state.pool).await {
        Ok(_) => {
            return StatusCode::CREATED.into_response();
        }
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
