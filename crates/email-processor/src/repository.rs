use anyhow::Result;
use chrono::{DateTime, Utc};
use shared::models::Attachment;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ParsedMessage {
    pub id: Uuid,
    pub account_id: Uuid,
    pub mailbox: String,
    pub message_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub from_address: String,
    pub from_name: Option<String>,
    pub to_addresses: serde_json::Value,
    pub cc_addresses: Option<serde_json::Value>,
    pub bcc_addresses: Option<serde_json::Value>,
    pub subject: Option<String>,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub has_attachments: bool,
    pub size_bytes: i32,
    pub received_at: DateTime<Utc>,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone)]
pub struct StoredEmail {
    pub account_id: Uuid,
    pub message_id: Uuid,
    pub mailbox: String,
    pub subject: Option<String>,
    pub from: Option<String>,
    pub received_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct EmailRepository {
    db: PgPool,
}

impl EmailRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn find_account_id_by_address(&self, address: &str) -> Result<Option<Uuid>> {
        let row = sqlx::query(
            r#"
            SELECT id
            FROM accounts
            WHERE address = $1 AND is_active = true
            "#,
        )
        .bind(address)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|row| row.get("id")))
    }

    pub async fn store_message(&self, message: ParsedMessage) -> Result<StoredEmail> {
        let mut tx = self.db.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO messages (
                id,
                account_id,
                message_id,
                in_reply_to,
                from_address,
                from_name,
                to_addresses,
                cc_addresses,
                bcc_addresses,
                subject,
                text_body,
                html_body,
                has_attachments,
                size_bytes,
                received_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(message.id)
        .bind(message.account_id)
        .bind(&message.message_id)
        .bind(&message.in_reply_to)
        .bind(&message.from_address)
        .bind(&message.from_name)
        .bind(&message.to_addresses)
        .bind(&message.cc_addresses)
        .bind(&message.bcc_addresses)
        .bind(&message.subject)
        .bind(&message.text_body)
        .bind(&message.html_body)
        .bind(message.has_attachments)
        .bind(message.size_bytes)
        .bind(message.received_at)
        .execute(&mut *tx)
        .await?;

        for attachment in &message.attachments {
            sqlx::query(
                r#"
                INSERT INTO attachments (
                    id,
                    message_id,
                    filename,
                    content_type,
                    size_bytes,
                    storage_key,
                    content_id
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(attachment.id)
            .bind(message.id)
            .bind(attachment.filename.as_deref().unwrap_or("attachment"))
            .bind(Some(&attachment.content_type))
            .bind(attachment.size_bytes as i32)
            .bind(&attachment.storage_key)
            .bind(Option::<String>::None)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(StoredEmail {
            account_id: message.account_id,
            message_id: message.id,
            mailbox: message.mailbox,
            subject: message.subject,
            from: Some(message.from_address),
            received_at: message.received_at,
        })
    }
}
