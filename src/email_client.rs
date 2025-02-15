use reqwest::Client;
use thiserror::Error;

use crate::domain::SubscriberEmail;

pub struct EmailClient<'a> {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail<'a>,
}

#[derive(Error, Debug)]
pub enum EmailClientError {
    #[error("some err")]
    GenericError,
}

impl<'a> EmailClient<'a> {
    pub fn new(url: String, sender: SubscriberEmail<'a>) -> Self {
        Self {
            http_client: Client::new(),
            base_url: url,
            sender,
        }
    }

    pub async fn send_email(
        recipient: SubscriberEmail<'a>,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), EmailClientError> {
        todo!()
    }
}
