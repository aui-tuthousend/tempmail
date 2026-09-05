use anyhow::{Context, Result};
use sqlx::{PgPool, Row};
use tokio::select;
use tokio::time;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::config::CleanupConfig;
use crate::object_storage::ObjectStorage;

pub struct CleanupWorker {
    config: CleanupConfig,
    db: PgPool,
    object_storage: ObjectStorage,
}

#[derive(Debug, Clone)]
struct PurgeCandidate {
    message_id: Uuid,
    storage_keys: Vec<String>,
}

impl CleanupWorker {
    pub fn new(config: CleanupConfig, db: PgPool, object_storage: ObjectStorage) -> Self {
        Self {
            config,
            db,
            object_storage,
        }
    }

    pub async fn run_until_shutdown(self) -> Result<()> {
        info!(
            interval_seconds = self.config.interval.as_secs(),
            soft_delete_retention_days = self.config.soft_delete_retention_days,
            "cleanup worker started"
        );
        self.run_once().await?;

        let mut interval = time::interval(self.config.interval);
        loop {
            select! {
                _ = interval.tick() => {
                    if let Err(error) = self.run_once().await {
                        error!(%error, "cleanup cycle failed");
                    }
                }
                signal = tokio::signal::ctrl_c() => {
                    signal.context("failed to listen for shutdown signal")?;
                    info!("cleanup worker shutdown requested");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn run_once(&self) -> Result<()> {
        let expired_sessions = self.delete_expired_sessions().await?;
        let expired_api_keys = self.disable_expired_api_keys().await?;
        let purged_messages = self.purge_soft_deleted_messages().await?;

        info!(
            expired_sessions,
            expired_api_keys, purged_messages, "cleanup cycle completed"
        );
        Ok(())
    }

    async fn delete_expired_sessions(&self) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM sessions
            WHERE expires_at <= NOW()
            "#,
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected())
    }

    async fn disable_expired_api_keys(&self) -> Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE api_keys
            SET is_active = false
            WHERE is_active = true
              AND expires_at IS NOT NULL
              AND expires_at <= NOW()
            "#,
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected())
    }

    async fn purge_soft_deleted_messages(&self) -> Result<u64> {
        let candidates = self.purge_candidates().await?;
        let mut purged = 0_u64;

        for candidate in candidates {
            if let Err(error) = self.purge_message(candidate).await {
                warn!(%error, "failed to purge soft-deleted message");
            } else {
                purged += 1;
            }
        }

        Ok(purged)
    }

    async fn purge_candidates(&self) -> Result<Vec<PurgeCandidate>> {
        let rows = sqlx::query(
            r#"
            SELECT
                m.id,
                COALESCE(array_agg(a.storage_key) FILTER (WHERE a.storage_key IS NOT NULL), ARRAY[]::TEXT[]) AS storage_keys
            FROM messages m
            LEFT JOIN attachments a ON a.message_id = m.id
            WHERE m.is_deleted = true
              AND m.updated_at < NOW() - ($1::INT * INTERVAL '1 day')
            GROUP BY m.id
            ORDER BY m.updated_at ASC
            LIMIT $2
            "#,
        )
        .bind(self.config.soft_delete_retention_days as i32)
        .bind(self.config.batch_size)
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| PurgeCandidate {
                message_id: row.get("id"),
                storage_keys: row.get("storage_keys"),
            })
            .collect())
    }

    async fn purge_message(&self, candidate: PurgeCandidate) -> Result<()> {
        for storage_key in &candidate.storage_keys {
            self.object_storage.delete_object(storage_key).await?;
        }

        let result = sqlx::query(
            r#"
            DELETE FROM messages
            WHERE id = $1
              AND is_deleted = true
              AND updated_at < NOW() - ($2::INT * INTERVAL '1 day')
            "#,
        )
        .bind(candidate.message_id)
        .bind(self.config.soft_delete_retention_days as i32)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            warn!(message_id = %candidate.message_id, "message skipped during purge");
        }

        Ok(())
    }
}
