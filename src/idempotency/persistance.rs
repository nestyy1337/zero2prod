use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "emailstatus")]
pub enum EmailStatus {
    Pending,
    Sent,
    Failed,
}

#[derive(Debug)]
pub struct EmailJob {
    pub user_id: Uuid,
    pub idempotency_key: String,
    pub status: EmailStatus,
    pub attempts: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[tracing::instrument(
    name = "Generate idempotency key",
    skip(subscriber_name, message_content)
)]
pub fn generate_idempotency_key(subscriber_name: &str, message_content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(subscriber_name.as_bytes());
    hasher.update(message_content.as_bytes());

    // Convert the hash to a hexadecimal string
    format!("{:x}", hasher.finalize())
}

#[tracing::instrument(name = "Get existing job", skip(pool, idempotency_key))]
pub async fn get_existing_job(
    pool: &PgPool,
    idempotency_key: &str,
) -> Result<Option<EmailJob>, sqlx::Error> {
    let job = sqlx::query_as!(
        EmailJob,
        r#"
        SELECT
            user_id,
            idempotency_key,
            status as "status: EmailStatus",
            attempts,
            created_at,
            updated_at
        FROM idempotency
        WHERE idempotency_key = $1
        "#,
        idempotency_key
    )
    .fetch_optional(pool)
    .await?;

    Ok(job)
}

#[tracing::instrument(name = "Update job status", skip(pool, idempotency_key))]
pub async fn update_job_status(
    pool: &PgPool,
    idempotency_key: &str,
    status: EmailStatus,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE idempotency
        SET status = $1, updated_at = $2
        WHERE idempotency_key = $3
        "#,
        status as EmailStatus,
        Utc::now(),
        idempotency_key
    )
    .execute(pool)
    .await?;

    Ok(())
}
