use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ses::{
    self as ses,
    types::{error::InvalidDeliveryOptionsException, Body, Content, Destination, Message},
};
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::json;
use ses::Client;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    domain::SubscriberEmail,
    idempotency::persistance::{update_job_status, EmailStatus},
    startup::AppState,
};

lazy_static::lazy_static! {
    static ref TEMPLATES: Handlebars<'static> = {
        let mut hb = Handlebars::new();
        hb.register_template_string("newsletter", include_str!("./templates/emails/newsletter.html"))
            .expect("Failed to register newsletter template");
        hb
    };
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EmailBody {
    pub title: String,
    pub message: String,
}

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

    pub async fn send_newsletter<'a>(
        &self,
        recipient: &str,
        subject: &str,
        name: &str,
        link: &str,
        key: &str,
        state: &AppState,
    ) -> Result<(), ses::Error> {
        let data = json!({
            "subscriber_name": &name,
            "link": &link,
            "unsubscribe_link": "#",
            "privacy_policy": "#",
        });

        let html = TEMPLATES
            .render("newsletter", &data)
            .expect("Failed to render newsletter template");

        let text = format!(
        "Hello {},\n\nThank you for subscribing to our newsletter.\n\nTo read our latest article, visit: {}\n\nTo unsubscribe, visit: #",
        name, link
    );

        let resp = self
            .ses_client
            .send_email()
            .source("activeandtoffi@gmail.com")
            .destination(Destination::builder().to_addresses(recipient).build())
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
        update_job_status(&state.pool, key, EmailStatus::Sent)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update job status: {:?}", e);
                aws_sdk_ses::Error::InvalidDeliveryOptionsException(
                    InvalidDeliveryOptionsException::builder()
                        .message("Failed to update job status")
                        .build(),
                )
            })?;
        Ok(())
    }
}
