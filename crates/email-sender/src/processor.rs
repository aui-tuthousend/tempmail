use anyhow::{anyhow, Context, Result};
use redis::aio::ConnectionManager;
use shared::events::{DomainEvent, EmailSentEvent, EMAIL_SENT_CHANNEL};
use shared::queue::{QueueMessage, SEND_EMAIL_PAYLOAD_FIELD};
use shared::redis_helper::publish_event;
use tokio::select;
use tracing::{error, info, warn};

use crate::config::EmailSenderConfig;
use crate::repository::EmailSenderRepository;
use crate::sender::send_outbox_message;
use crate::storage::ObjectStorage;

pub struct EmailSender {
    config: EmailSenderConfig,
    redis: ConnectionManager,
    repository: EmailSenderRepository,
    object_storage: ObjectStorage,
}

impl EmailSender {
    pub fn new(
        config: EmailSenderConfig,
        redis: ConnectionManager,
        repository: EmailSenderRepository,
        object_storage: ObjectStorage,
    ) -> Self {
        Self {
            config,
            redis,
            repository,
            object_storage,
        }
    }

    pub async fn run_until_shutdown(mut self) -> Result<()> {
        ensure_send_stream_group(&mut self.redis, &self.config).await?;
        info!(
            stream = %self.config.queue.send_email_stream,
            group = %self.config.consumer_group,
            consumer = %self.config.consumer_name,
            "email sender started"
        );

        loop {
            select! {
                result = self.process_next_batch() => match result {
                    Ok(()) => {},
                    Err(e) => error!(%e, "batch processing error"),
                },
                signal = tokio::signal::ctrl_c() => {
                    signal.context("failed to listen for shutdown signal")?;
                    info!("email sender shutdown requested");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn process_next_batch(&mut self) -> Result<()> {
        let entries = read_send_group_batch(&mut self.redis, &self.config).await?;

        for entry in entries {
            let result = self.process_entry(&entry).await;
            match result {
                Ok(()) => {}
                Err(e) => error!(stream_id = %entry.id, %e, "failed to send email"),
            }

            ack_message(&mut self.redis, &self.config, &entry.id).await?;
        }

        Ok(())
    }

    async fn process_entry(&mut self, entry: &StreamEntry) -> Result<()> {
        let message: QueueMessage = serde_json::from_str(&entry.payload)?;
        let QueueMessage::SendEmail(send_msg) = message else {
            return Err(anyhow!("unexpected raw email queue message"));
        };

        let Some(record) = self.repository.find_by_id(send_msg.outbox_id).await? else {
            warn!(
                outbox_id = %send_msg.outbox_id,
                "outbox message not found or already processed, skipping"
            );
            return Ok(());
        };

        let attachments = self.repository.find_attachments(send_msg.outbox_id).await?;

        match send_outbox_message(
            &record,
            &attachments,
            &self.repository,
            &self.object_storage,
            &self.config.smtp,
        )
        .await
        {
            Ok(()) => {
                self.repository
                    .update_status(record.id, "sent", None)
                    .await?;

                let event = DomainEvent::EmailSent(EmailSentEvent {
                    account_id: record.account_id,
                    outbox_id: record.id,
                    status: "sent".to_owned(),
                });
                publish_event(&mut self.redis, EMAIL_SENT_CHANNEL, &event).await?;

                info!(
                    outbox_id = %record.id,
                    account_id = %record.account_id,
                    "email sent successfully"
                );
            }
            Err(e) => {
                let err_str = e.to_string();
                self.repository
                    .update_status(record.id, "failed", Some(&err_str))
                    .await?;

                let event = DomainEvent::EmailSent(EmailSentEvent {
                    account_id: record.account_id,
                    outbox_id: record.id,
                    status: "failed".to_owned(),
                });
                publish_event(&mut self.redis, EMAIL_SENT_CHANNEL, &event).await?;

                warn!(
                    outbox_id = %record.id,
                    account_id = %record.account_id,
                    %e,
                    "email send failed"
                );
            }
        }

        Ok(())
    }
}

async fn ensure_send_stream_group(
    redis: &mut ConnectionManager,
    config: &EmailSenderConfig,
) -> Result<()> {
    let result: redis::RedisResult<()> = redis::cmd("XGROUP")
        .arg("CREATE")
        .arg(&config.queue.send_email_stream)
        .arg(&config.consumer_group)
        .arg("0")
        .arg("MKSTREAM")
        .query_async(redis)
        .await;

    match result {
        Ok(()) => Ok(()),
        Err(error) if error.to_string().contains("BUSYGROUP") => Ok(()),
        Err(error) => Err(error.into()),
    }
}

async fn read_send_group_batch(
    redis: &mut ConnectionManager,
    config: &EmailSenderConfig,
) -> Result<Vec<StreamEntry>> {
    let value: redis::Value = redis::cmd("XREADGROUP")
        .arg("GROUP")
        .arg(&config.consumer_group)
        .arg(&config.consumer_name)
        .arg("COUNT")
        .arg(config.batch_size)
        .arg("BLOCK")
        .arg(config.queue.stream_block_ms)
        .arg("STREAMS")
        .arg(&config.queue.send_email_stream)
        .arg(">")
        .query_async(redis)
        .await?;

    parse_stream_entries(value)
}

fn parse_stream_entries(value: redis::Value) -> Result<Vec<StreamEntry>> {
    let redis::Value::Bulk(streams) = value else {
        return Ok(Vec::new());
    };

    let mut entries = Vec::new();
    for stream in streams {
        let redis::Value::Bulk(stream_items) = stream else {
            continue;
        };
        if stream_items.len() != 2 {
            continue;
        }

        let redis::Value::Bulk(messages) = &stream_items[1] else {
            continue;
        };

        for message in messages {
            let Some(entry) = parse_stream_entry(message)? else {
                continue;
            };
            entries.push(entry);
        }
    }

    Ok(entries)
}

fn parse_stream_entry(value: &redis::Value) -> Result<Option<StreamEntry>> {
    let redis::Value::Bulk(items) = value else {
        return Ok(None);
    };
    if items.len() != 2 {
        return Ok(None);
    }

    let id = value_to_string(&items[0]).context("Redis Stream entry id is not a string")?;
    let redis::Value::Bulk(fields) = &items[1] else {
        return Ok(None);
    };

    let mut payload = None;
    for field in fields.chunks(2) {
        if field.len() != 2 {
            continue;
        }

        let Some(name) = value_to_string(&field[0]) else {
            continue;
        };
        if name == SEND_EMAIL_PAYLOAD_FIELD {
            payload = value_to_string(&field[1]);
            break;
        }
    }

    Ok(payload.map(|payload| StreamEntry { id, payload }))
}

fn value_to_string(value: &redis::Value) -> Option<String> {
    match value {
        redis::Value::Data(bytes) => String::from_utf8(bytes.clone()).ok(),
        redis::Value::Status(text) => Some(text.clone()),
        redis::Value::Okay => Some("OK".to_owned()),
        _ => None,
    }
}

async fn ack_message(
    redis: &mut ConnectionManager,
    config: &EmailSenderConfig,
    id: &str,
) -> Result<()> {
    let _: usize = redis::cmd("XACK")
        .arg(&config.queue.send_email_stream)
        .arg(&config.consumer_group)
        .arg(id)
        .query_async(redis)
        .await?;
    Ok(())
}

struct StreamEntry {
    id: String,
    payload: String,
}
