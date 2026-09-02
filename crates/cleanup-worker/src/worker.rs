use anyhow::{Context, Result};
use chrono::Utc;
use redis::aio::ConnectionManager;
use shared::keys::{mailbox_index_key, message_expiry_index_key, message_key};
use shared::models::EmailMessage;
use shared::redis_helper::{delete_key, get_json};
use tokio::select;
use tokio::time;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::config::CleanupConfig;
use crate::object_storage::ObjectStorage;

pub struct CleanupWorker {
    config: CleanupConfig,
    redis: ConnectionManager,
    object_storage: ObjectStorage,
}

impl CleanupWorker {
    pub fn new(
        config: CleanupConfig,
        redis: ConnectionManager,
        object_storage: ObjectStorage,
    ) -> Self {
        Self {
            config,
            redis,
            object_storage,
        }
    }

    pub async fn run_until_shutdown(mut self) -> Result<()> {
        info!(
            interval_seconds = self.config.interval.as_secs(),
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

    async fn run_once(&mut self) -> Result<()> {
        let members = expired_members(&mut self.redis, self.config.batch_size).await?;
        if members.is_empty() {
            return Ok(());
        }

        let mut deleted = 0_u64;
        for member in members {
            match self.cleanup_member(&member).await {
                Ok(()) => deleted += 1,
                Err(error) => warn!(%error, %member, "failed to cleanup expired message"),
            }
        }

        info!(deleted, "cleanup cycle completed");
        Ok(())
    }

    async fn cleanup_member(&mut self, member: &str) -> Result<()> {
        let Some((mailbox, message_id)) = parse_expiry_member(member) else {
            remove_expiry_member(&mut self.redis, member).await?;
            return Ok(());
        };

        let key = message_key(&mailbox, message_id);
        let message = get_json::<EmailMessage>(&mut self.redis, &key).await?;
        if let Some(message) = &message {
            for attachment in &message.attachments {
                self.object_storage
                    .delete_object(&attachment.storage_key)
                    .await?;
            }
        }

        delete_key(&mut self.redis, &key).await?;
        remove_mailbox_index_entry(&mut self.redis, &mailbox, message_id).await?;
        remove_expiry_member(&mut self.redis, member).await?;
        Ok(())
    }
}

async fn expired_members(redis: &mut ConnectionManager, batch_size: usize) -> Result<Vec<String>> {
    let now = Utc::now().timestamp();
    let members: Vec<String> = redis::cmd("ZRANGEBYSCORE")
        .arg(message_expiry_index_key())
        .arg("-inf")
        .arg(now)
        .arg("LIMIT")
        .arg(0)
        .arg(batch_size)
        .query_async(redis)
        .await?;
    Ok(members)
}

async fn remove_mailbox_index_entry(
    redis: &mut ConnectionManager,
    mailbox: &str,
    message_id: Uuid,
) -> Result<()> {
    let _: usize = redis::cmd("LREM")
        .arg(mailbox_index_key(mailbox))
        .arg(0)
        .arg(message_id.to_string())
        .query_async(redis)
        .await?;
    Ok(())
}

async fn remove_expiry_member(redis: &mut ConnectionManager, member: &str) -> Result<()> {
    let _: usize = redis::cmd("ZREM")
        .arg(message_expiry_index_key())
        .arg(member)
        .query_async(redis)
        .await?;
    Ok(())
}

fn parse_expiry_member(member: &str) -> Option<(String, Uuid)> {
    let (mailbox, message_id) = member.rsplit_once('|')?;
    Some((mailbox.to_owned(), message_id.parse().ok()?))
}
