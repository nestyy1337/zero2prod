use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ses::{
    self as ses,
    types::{Body, Content, Destination, Message},
};
use aws_types::region::Region;
use serde::{Deserialize, Serialize};
use ses::Client;
use thiserror::Error;

use crate::domain::SubscriberEmail;

#[derive(Debug, Clone)]
pub struct EmailClient {
    ses_client: Client,
    // base_url: String,
    // sender: SubscriberEmail<'a>,
}

#[derive(Debug, Clone, Serialize)]
struct EmailContents<'a> {
    #[serde(rename = "From")]
    sender: String,
    #[serde(rename = "To")]
    recipient: SubscriberEmail<'a>,
    #[serde(rename = "Subject")]
    subject: &'a str,
    #[serde(rename = "TextBody")]
    text_content: &'a str,
    #[serde(rename = "HtmlBody")]
    html_content: &'a str,
}

impl<'a> EmailContents<'a> {
    pub fn new(
        recipient: SubscriberEmail<'a>,
        subject: &'a str,
        // html_content: &'a str,
        // text_content: &'a str,
    ) -> Self {
        Self {
            sender: "szymon.gluch@netxp.pl".to_string(),
            recipient,
            subject,
            text_content: "Hello dear Postmark user.",
            html_content: "<html><body><strong>Hello</strong> dear Postmark user.</body></html>",
        }
    }
}

#[derive(Error, Debug)]
pub enum EmailClientError {
    #[error("some err")]
    GenericError,
}

impl EmailClient {
    pub fn new(client: Client) -> Self {
        Self { ses_client: client }
    }

    pub async fn send_email_example<'a>(
        &self,
        recipient: &SubscriberEmail<'a>,
        subject: &str,
        html: &str,
        text: &str,
    ) -> Result<(), ses::Error> {
        let resp = self
            .ses_client
            .send_email()
            .source("activeandtoffi@gmail.com")
            .destination(
                Destination::builder()
                    .to_addresses(recipient.as_ref())
                    .build(),
            )
            .message(
                Message::builder()
                    .subject(Content::builder().data(subject).build().unwrap())
                    .body(
                        Body::builder()
                            .html(Content::builder().data(html).build().unwrap())
                            .text(Content::builder().data(text).build().unwrap())
                            .build(),
                    )
                    .build(),
            )
            .send()
            .await?;
        tracing::info!("Email sent: {:?}", resp);
        Ok(())
    }
}
