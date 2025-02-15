use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

use super::{ParseError, SubscriberEmail, SubscriberName};

#[derive(Debug)]
pub struct Subscriber<'a> {
    name: SubscriberName,
    email: SubscriberEmail<'a>,
}

impl<'a> Subscriber<'a> {
    pub fn new(name: &str, email: &'a str) -> Result<Subscriber<'a>, ParseError> {
        let name = SubscriberName::parse(name)?;
        let email = SubscriberEmail::parse(email)?;

        Ok(Subscriber { name, email })
    }

    #[tracing::instrument(
    name = "Inserting a new subscriber",
    skip(pool),
    fields(
    subscriber_email = %self.email,
    subscriber_name = %self.name
))]
    pub async fn try_insert(&self, pool: &PgPool) -> Result<(), sqlx::Error> {
        let query_span = tracing::info_span!("Saving new subscriber details in the database");
        let _query = sqlx::query!(
            r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
            Uuid::new_v4(),
            self.email.as_ref(),
            self.name.as_ref(),
            Utc::now()
        )
        .execute(pool)
        .instrument(query_span)
        .await?;
        Ok(())
    }
}
