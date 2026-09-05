use chrono::{DateTime, Utc};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use shared::keys::{mailbox_index_key, mailbox_key, message_key};
use shared::models::{Account, AccountCredentials, EmailMessage, Mailbox, Session, StoredMessage};
use shared::redis_helper::{get_json, set_json};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::dto::SessionAccountResponse;

#[derive(Clone)]
pub struct MailboxRepository {
    redis: ConnectionManager,
}

impl MailboxRepository {
    pub fn new(redis: ConnectionManager) -> Self {
        Self { redis }
    }

    pub async fn store_mailbox(&self, mailbox: &Mailbox, ttl_seconds: u64) -> anyhow::Result<()> {
        let mut redis = self.redis.clone();
        let key = mailbox_key(&mailbox.address.as_string());
        set_json(&mut redis, &key, mailbox, ttl_seconds).await?;
        Ok(())
    }

    pub async fn list_messages(&self, mailbox: &str) -> anyhow::Result<Vec<EmailMessage>> {
        let mut redis = self.redis.clone();
        let message_ids = mailbox_message_ids(&mut redis, mailbox).await?;
        let mut messages = Vec::new();

        for message_id in message_ids {
            let key = message_key(mailbox, message_id);
            if let Some(message) = get_json::<EmailMessage>(&mut redis, &key).await? {
                messages.push(message);
            }
        }

        Ok(messages)
    }
}

async fn mailbox_message_ids(
    redis: &mut redis::aio::ConnectionManager,
    mailbox: &str,
) -> anyhow::Result<Vec<Uuid>> {
    let ids: Vec<String> = redis.lrange(mailbox_index_key(mailbox), 0, -1).await?;
    Ok(ids
        .into_iter()
        .filter_map(|id| id.parse::<Uuid>().ok())
        .collect())
}

#[derive(Clone)]
pub struct ApiKeyRepository {
    db: PgPool,
}

impl ApiKeyRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn find_active_by_hash(&self, key_hash: &str) -> Result<Option<Uuid>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id
            FROM api_keys
            WHERE key_hash = $1
              AND is_active = true
              AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .bind(key_hash)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|row| row.get("id")))
    }

    pub async fn touch_last_used(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE api_keys
            SET last_used_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct NewAccount {
    pub id: Uuid,
    pub username: String,
    pub local_part: String,
    pub domain: String,
    pub password_hash: String,
    pub display_name: Option<String>,
}

#[derive(Clone)]
pub struct AccountRepository {
    db: PgPool,
}

impl AccountRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn create(&self, account: NewAccount) -> Result<Account, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO accounts (
                id,
                username,
                local_part,
                domain,
                password_hash,
                display_name
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING
                id,
                username,
                local_part,
                domain,
                address,
                display_name,
                is_active,
                created_at,
                updated_at
            "#,
        )
        .bind(account.id)
        .bind(account.username)
        .bind(account.local_part)
        .bind(account.domain)
        .bind(account.password_hash)
        .bind(account.display_name)
        .fetch_one(&self.db)
        .await?;

        Ok(account_from_row(&row))
    }
}

#[derive(Debug, Clone)]
pub struct NewSession {
    pub id: Uuid,
    pub token_hash: String,
    pub device_info: Option<serde_json::Value>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct SessionRepository {
    db: PgPool,
}

impl SessionRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn find_account_credentials(
        &self,
        username_or_email: &str,
    ) -> Result<Option<AccountCredentials>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, username, address, password_hash, is_active
            FROM accounts
            WHERE username = $1 OR address = $1
            "#,
        )
        .bind(username_or_email)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|row| AccountCredentials {
            id: row.get("id"),
            username: row.get("username"),
            address: row.get("address"),
            password_hash: row.get("password_hash"),
            is_active: row.get("is_active"),
        }))
    }

    pub async fn find_valid_session_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<Session>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, token_hash, device_info, expires_at, created_at, last_active_at
            FROM sessions
            WHERE token_hash = $1 AND expires_at > NOW()
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.map(|row| Session {
            id: row.get("id"),
            token_hash: row.get("token_hash"),
            device_info: row.get("device_info"),
            expires_at: row.get("expires_at"),
            created_at: row.get("created_at"),
            last_active_at: row.get("last_active_at"),
        }))
    }

    pub async fn create_session(&self, session: NewSession) -> Result<Session, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO sessions (id, token_hash, device_info, expires_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, token_hash, device_info, expires_at, created_at, last_active_at
            "#,
        )
        .bind(session.id)
        .bind(session.token_hash)
        .bind(session.device_info)
        .bind(session.expires_at)
        .fetch_one(&self.db)
        .await?;

        Ok(Session {
            id: row.get("id"),
            token_hash: row.get("token_hash"),
            device_info: row.get("device_info"),
            expires_at: row.get("expires_at"),
            created_at: row.get("created_at"),
            last_active_at: row.get("last_active_at"),
        })
    }

    pub async fn touch_session(&self, session_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE sessions SET last_active_at = NOW() WHERE id = $1")
            .bind(session_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    pub async fn attach_account_and_activate_if_needed(
        &self,
        session_id: Uuid,
        account_id: Uuid,
        session_account_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.db.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO session_accounts (id, session_id, account_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (session_id, account_id) DO NOTHING
            "#,
        )
        .bind(session_account_id)
        .bind(session_id)
        .bind(account_id)
        .execute(&mut *tx)
        .await?;

        let active_exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM session_accounts
                WHERE session_id = $1 AND is_active = true
            )
            "#,
        )
        .bind(session_id)
        .fetch_one(&mut *tx)
        .await?;

        if !active_exists {
            sqlx::query(
                r#"
                UPDATE session_accounts
                SET is_active = true
                WHERE session_id = $1 AND account_id = $2
                "#,
            )
            .bind(session_id)
            .bind(account_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn list_accounts(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<SessionAccountResponse>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                a.id AS account_id,
                a.username,
                a.address,
                a.display_name,
                sa.is_active,
                sa.logged_in_at
            FROM session_accounts sa
            JOIN accounts a ON a.id = sa.account_id
            WHERE sa.session_id = $1
            ORDER BY sa.logged_in_at ASC
            "#,
        )
        .bind(session_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| SessionAccountResponse {
                account_id: row.get("account_id"),
                username: row.get("username"),
                address: row.get("address"),
                display_name: row.get("display_name"),
                is_active: row.get("is_active"),
                logged_in_at: row.get("logged_in_at"),
            })
            .collect())
    }

    pub async fn activate_account(
        &self,
        session_id: Uuid,
        account_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let mut tx = self.db.begin().await?;

        let attached: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM session_accounts
                WHERE session_id = $1 AND account_id = $2
            )
            "#,
        )
        .bind(session_id)
        .bind(account_id)
        .fetch_one(&mut *tx)
        .await?;

        if !attached {
            tx.rollback().await?;
            return Ok(false);
        }

        sqlx::query("UPDATE session_accounts SET is_active = false WHERE session_id = $1")
            .bind(session_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r#"
            UPDATE session_accounts
            SET is_active = true
            WHERE session_id = $1 AND account_id = $2
            "#,
        )
        .bind(session_id)
        .bind(account_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(true)
    }
}

fn account_from_row(row: &sqlx::postgres::PgRow) -> Account {
    Account {
        id: row.get("id"),
        username: row.get("username"),
        local_part: row.get("local_part"),
        domain: row.get("domain"),
        address: row.get("address"),
        display_name: row.get("display_name"),
        is_active: row.get("is_active"),
        created_at: row.get::<DateTime<Utc>, _>("created_at"),
        updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    }
}

#[derive(Debug, Clone, Default)]
pub struct MessageFlagsUpdate {
    pub is_read: Option<bool>,
    pub is_starred: Option<bool>,
    pub is_archived: Option<bool>,
    pub is_deleted: Option<bool>,
}

#[derive(Clone)]
pub struct MessageRepository {
    db: PgPool,
}

impl MessageRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn active_account_id(&self, session_id: Uuid) -> Result<Option<Uuid>, sqlx::Error> {
        sqlx::query_scalar(
            r#"
            SELECT account_id
            FROM session_accounts
            WHERE session_id = $1 AND is_active = true
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.db)
        .await
    }

    pub async fn list_inbox(&self, account_id: Uuid) -> Result<Vec<StoredMessage>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
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
                is_read,
                is_starred,
                is_archived,
                is_deleted,
                has_attachments,
                size_bytes,
                received_at,
                created_at,
                updated_at
            FROM messages
            WHERE account_id = $1
              AND is_deleted = false
              AND is_archived = false
            ORDER BY received_at DESC
            "#,
        )
        .bind(account_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rows.iter().map(stored_message_from_row).collect())
    }

    pub async fn find_by_id_for_account(
        &self,
        message_id: Uuid,
        account_id: Uuid,
    ) -> Result<Option<StoredMessage>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
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
                is_read,
                is_starred,
                is_archived,
                is_deleted,
                has_attachments,
                size_bytes,
                received_at,
                created_at,
                updated_at
            FROM messages
            WHERE id = $1 AND account_id = $2
            "#,
        )
        .bind(message_id)
        .bind(account_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.as_ref().map(stored_message_from_row))
    }

    pub async fn update_flags_for_account(
        &self,
        message_id: Uuid,
        account_id: Uuid,
        update: MessageFlagsUpdate,
    ) -> Result<Option<StoredMessage>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            UPDATE messages
            SET
                is_read = COALESCE($3, is_read),
                is_starred = COALESCE($4, is_starred),
                is_archived = COALESCE($5, is_archived),
                is_deleted = COALESCE($6, is_deleted),
                updated_at = NOW()
            WHERE id = $1 AND account_id = $2
            RETURNING
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
                is_read,
                is_starred,
                is_archived,
                is_deleted,
                has_attachments,
                size_bytes,
                received_at,
                created_at,
                updated_at
            "#,
        )
        .bind(message_id)
        .bind(account_id)
        .bind(update.is_read)
        .bind(update.is_starred)
        .bind(update.is_archived)
        .bind(update.is_deleted)
        .fetch_optional(&self.db)
        .await?;

        Ok(row.as_ref().map(stored_message_from_row))
    }
}

fn stored_message_from_row(row: &sqlx::postgres::PgRow) -> StoredMessage {
    StoredMessage {
        id: row.get("id"),
        account_id: row.get("account_id"),
        message_id: row.get("message_id"),
        in_reply_to: row.get("in_reply_to"),
        from_address: row.get("from_address"),
        from_name: row.get("from_name"),
        to_addresses: row.get("to_addresses"),
        cc_addresses: row.get("cc_addresses"),
        bcc_addresses: row.get("bcc_addresses"),
        subject: row.get("subject"),
        text_body: row.get("text_body"),
        html_body: row.get("html_body"),
        is_read: row.get("is_read"),
        is_starred: row.get("is_starred"),
        is_archived: row.get("is_archived"),
        is_deleted: row.get("is_deleted"),
        has_attachments: row.get("has_attachments"),
        size_bytes: row.get("size_bytes"),
        received_at: row.get("received_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[derive(Clone)]
pub struct HealthRepository {
    db: PgPool,
}

impl HealthRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn ping_db(&self) -> anyhow::Result<()> {
        sqlx::query("SELECT 1").execute(&self.db).await?;
        Ok(())
    }
}
