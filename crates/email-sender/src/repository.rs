use anyhow::Result;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct OutboxAttachmentRow {
    pub id: Uuid,
    pub filename: String,
    pub content_type: Option<String>,
    pub size_bytes: i32,
    pub storage_key: String,
    pub content_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OutboxMessageRecord {
    pub id: Uuid,
    pub account_id: Uuid,
    pub to_addresses: serde_json::Value,
    pub cc_addresses: Option<serde_json::Value>,
    pub bcc_addresses: Option<serde_json::Value>,
    pub subject: Option<String>,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub has_attachments: bool,
    pub size_bytes: i32,
    pub in_reply_to: Option<String>,
}

pub struct StoredOutboxStatus {
    pub id: Uuid,
    pub account_id: Uuid,
    pub status: String,
}

#[derive(Clone)]
pub struct EmailSenderRepository {
    db: PgPool,
}

impl EmailSenderRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn find_by_id(&self, outbox_id: Uuid) -> Result<Option<OutboxMessageRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                id, account_id, to_addresses, cc_addresses, bcc_addresses,
                subject, text_body, html_body, has_attachments, size_bytes, in_reply_to
            FROM outbox_messages
            WHERE id = $1 AND status = 'pending'
            "#,
        )
        .bind(outbox_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|r: sqlx::postgres::PgRow| OutboxMessageRecord {
            id: r.get("id"),
            account_id: r.get("account_id"),
            to_addresses: r.get("to_addresses"),
            cc_addresses: r.get("cc_addresses"),
            bcc_addresses: r.get("bcc_addresses"),
            subject: r.get("subject"),
            text_body: r.get("text_body"),
            html_body: r.get("html_body"),
            has_attachments: r.get("has_attachments"),
            size_bytes: r.get("size_bytes"),
            in_reply_to: r.get("in_reply_to"),
        }))
    }

    pub async fn find_attachments(&self, outbox_id: Uuid) -> Result<Vec<OutboxAttachmentRow>> {
        let rows = sqlx::query(
            r#"
            SELECT id, filename, content_type, size_bytes, storage_key, content_id
            FROM outbox_attachments
            WHERE outbox_id = $1
            "#,
        )
        .bind(outbox_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .iter()
            .map(|r: &sqlx::postgres::PgRow| OutboxAttachmentRow {
                id: r.get("id"),
                filename: r.get("filename"),
                content_type: r.get("content_type"),
                size_bytes: r.get("size_bytes"),
                storage_key: r.get("storage_key"),
                content_id: r.get("content_id"),
            })
            .collect())
    }

    pub async fn update_status(
        &self,
        outbox_id: Uuid,
        status: &str,
        error_message: Option<&str>,
    ) -> Result<Option<StoredOutboxStatus>> {
        let row = sqlx::query(
            r#"
            UPDATE outbox_messages
            SET status = $2,
                error_message = $3,
                sent_at = CASE WHEN $2 = 'sent' THEN NOW() ELSE sent_at END,
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, account_id, status
            "#,
        )
        .bind(outbox_id)
        .bind(status)
        .bind(error_message)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|r: sqlx::postgres::PgRow| StoredOutboxStatus {
            id: r.get("id"),
            account_id: r.get("account_id"),
            status: r.get("status"),
        }))
    }
}
