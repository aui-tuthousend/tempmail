use anyhow::{anyhow, Context, Result};
use chrono::Duration;
use mail_parser::{Addr, Address, MessageParser, MimeHeaders};
use redis::aio::ConnectionManager;
use shared::events::{DomainEvent, EmailReceivedEvent, EMAIL_RECEIVED_CHANNEL};
use shared::keys::{mailbox_index_key, message_expiry_index_key, message_key};
use shared::models::{EmailMessage, RawEmail};
use shared::queue::{QueueMessage, RAW_EMAIL_PAYLOAD_FIELD};
use shared::redis_helper::{publish_event, set_json};
use tokio::select;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::config::EmailProcessorConfig;
use crate::storage::ObjectStorage;

pub struct EmailProcessor {
    config: EmailProcessorConfig,
    redis: ConnectionManager,
    object_storage: ObjectStorage,
}

impl EmailProcessor {
    pub fn new(
        config: EmailProcessorConfig,
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
        ensure_consumer_group(&mut self.redis, &self.config).await?;
        info!(
            stream = %self.config.queue.raw_email_stream,
            group = %self.config.queue.consumer_group,
            consumer = %self.config.consumer_name,
            "email processor started"
        );

        loop {
            select! {
                result = self.process_next_batch() => result?,
                signal = tokio::signal::ctrl_c() => {
                    signal.context("failed to listen for shutdown signal")?;
                    info!("email processor shutdown requested");
                    break;
                }
            }
        }

        Ok(())
    }

    async fn process_next_batch(&mut self) -> Result<()> {
        let entries = read_group_batch(&mut self.redis, &self.config).await?;

        for entry in entries {
            if let Err(error) = self.process_entry(&entry).await {
                error!(stream_id = %entry.id, %error, "failed to process raw email");
                continue;
            }

            ack_message(&mut self.redis, &self.config, &entry.id).await?;
        }

        Ok(())
    }

    async fn process_entry(&mut self, entry: &StreamEntry) -> Result<()> {
        let message: QueueMessage = serde_json::from_str(&entry.payload)?;
        let QueueMessage::RawEmail(raw_message) = message;

        let email = build_email_message(
            raw_message.email,
            self.config.mailbox.ttl_seconds,
            &self.object_storage,
        )
        .await?;

        let message_key = message_key(&email.mailbox, email.id);
        set_json(
            &mut self.redis,
            &message_key,
            &email,
            self.config.mailbox.ttl_seconds + self.config.message_ttl_grace_seconds,
        )
        .await?;

        let index_key = mailbox_index_key(&email.mailbox);
        add_to_mailbox_index(
            &mut self.redis,
            &index_key,
            email.id,
            self.config.mailbox.ttl_seconds + self.config.message_ttl_grace_seconds,
        )
        .await?;
        add_to_expiry_index(&mut self.redis, &email).await?;

        let event = DomainEvent::EmailReceived(EmailReceivedEvent {
            message_id: email.id,
            mailbox: email.mailbox.clone(),
            subject: email.subject.clone(),
            from: email.from.clone(),
            received_at: email.received_at,
        });
        publish_event(&mut self.redis, EMAIL_RECEIVED_CHANNEL, &event).await?;

        info!(
            stream_id = %entry.id,
            message_id = %email.id,
            mailbox = %email.mailbox,
            "email processed"
        );
        Ok(())
    }
}

async fn ensure_consumer_group(
    redis: &mut ConnectionManager,
    config: &EmailProcessorConfig,
) -> Result<()> {
    let result: redis::RedisResult<()> = redis::cmd("XGROUP")
        .arg("CREATE")
        .arg(&config.queue.raw_email_stream)
        .arg(&config.queue.consumer_group)
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

async fn read_group_batch(
    redis: &mut ConnectionManager,
    config: &EmailProcessorConfig,
) -> Result<Vec<StreamEntry>> {
    let value: redis::Value = redis::cmd("XREADGROUP")
        .arg("GROUP")
        .arg(&config.queue.consumer_group)
        .arg(&config.consumer_name)
        .arg("COUNT")
        .arg(config.batch_size)
        .arg("BLOCK")
        .arg(config.queue.stream_block_ms)
        .arg("STREAMS")
        .arg(&config.queue.raw_email_stream)
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
        if name == RAW_EMAIL_PAYLOAD_FIELD {
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
    config: &EmailProcessorConfig,
    id: &str,
) -> Result<()> {
    let _: usize = redis::cmd("XACK")
        .arg(&config.queue.raw_email_stream)
        .arg(&config.queue.consumer_group)
        .arg(id)
        .query_async(redis)
        .await?;
    Ok(())
}

async fn add_to_mailbox_index(
    redis: &mut ConnectionManager,
    index_key: &str,
    message_id: Uuid,
    ttl_seconds: u64,
) -> Result<()> {
    let _: usize = redis::cmd("LPUSH")
        .arg(index_key)
        .arg(message_id.to_string())
        .query_async(redis)
        .await?;
    let _: bool = redis::cmd("EXPIRE")
        .arg(index_key)
        .arg(ttl_seconds)
        .query_async(redis)
        .await?;
    Ok(())
}

async fn add_to_expiry_index(redis: &mut ConnectionManager, email: &EmailMessage) -> Result<()> {
    let member = format!("{}|{}", email.mailbox, email.id);
    let score = email.expires_at.timestamp();
    let _: usize = redis::cmd("ZADD")
        .arg(message_expiry_index_key())
        .arg(score)
        .arg(member)
        .query_async(redis)
        .await?;
    Ok(())
}

async fn build_email_message(
    raw: RawEmail,
    ttl_seconds: u64,
    object_storage: &ObjectStorage,
) -> Result<EmailMessage> {
    let parsed = MessageParser::default()
        .parse(&raw.data)
        .ok_or_else(|| anyhow!("failed to parse raw email"))?;

    let message_id = Uuid::new_v4();
    let mailbox = raw
        .envelope
        .rcpt_to
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("raw email has no recipient"))?;
    let expires_at = raw.received_at + Duration::seconds(ttl_seconds as i64);
    let text_body = parsed.body_text(0).map(|body| body.into_owned());
    let html_body = parsed.body_html(0).map(|body| body.into_owned());
    let mut attachments = Vec::new();

    for part in parsed.attachments() {
        let content_type = part
            .content_type()
            .map(content_type_to_string)
            .unwrap_or_else(|| "application/octet-stream".to_owned());
        let filename = part
            .content_disposition()
            .and_then(|value| value.attribute("filename"))
            .or_else(|| {
                part.content_type()
                    .and_then(|value| value.attribute("name"))
            })
            .map(ToOwned::to_owned);

        match object_storage
            .put_attachment(message_id, filename, content_type, part.contents().to_vec())
            .await?
        {
            Some(attachment) => attachments.push(attachment),
            None => {
                warn!(message_id = %message_id, "attachment skipped because R2 is not configured")
            }
        }
    }

    Ok(EmailMessage {
        id: message_id,
        mailbox,
        from: parsed.from().and_then(address_to_string),
        to: parsed
            .to()
            .map(addresses_to_strings)
            .unwrap_or_else(|| raw.envelope.rcpt_to.clone()),
        subject: parsed.subject().map(ToOwned::to_owned),
        text_body,
        html_body,
        attachments,
        received_at: raw.received_at,
        expires_at,
    })
}

fn content_type_to_string(content_type: &mail_parser::ContentType<'_>) -> String {
    match content_type.subtype() {
        Some(subtype) => format!("{}/{}", content_type.ctype(), subtype),
        None => content_type.ctype().to_owned(),
    }
}

fn address_to_string(address: &Address<'_>) -> Option<String> {
    address.iter().find_map(addr_to_string)
}

fn addresses_to_strings(addresses: &Address<'_>) -> Vec<String> {
    addresses.iter().filter_map(addr_to_string).collect()
}

fn addr_to_string(addr: &Addr<'_>) -> Option<String> {
    addr.address().map(ToOwned::to_owned)
}

struct StreamEntry {
    id: String,
    payload: String,
}
