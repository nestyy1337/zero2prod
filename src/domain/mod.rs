use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use thiserror::Error;

mod new_subscriber;
mod subscriber_email;
mod subscriber_name;

pub use new_subscriber::Subscriber;
pub use subscriber_email::SubscriberEmail;
pub use subscriber_name::SubscriberName;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Name too long")]
    TooLong,
    #[error("Empty or whitespace")]
    Empty,
    #[error("Forbidden character")]
    ForbiddenChar,
    #[error("Bad Name")]
    BadName,
    #[error("Bad Email")]
    BadEmail,
}

impl From<ParseError> for Response {
    fn from(e: ParseError) -> Self {
        (StatusCode::BAD_REQUEST, e.to_string()).into_response()
    }
}
