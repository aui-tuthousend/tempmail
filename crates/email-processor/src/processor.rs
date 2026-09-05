use anyhow::{anyhow, Context, Result};
use chrono::Duration;
use mail_parser::{Addr, Address, MessageParser, MimeHeaders};
use redis::aio::ConnectionManager;
use shared::events::{DomainEvent, EmailReceivedEvent, EMAIL_RECEIVED_CHANNEL};
use shared::ids::new_uuid_v7;
use shared::models::RawEmail;
use shared::queue::{QueueMessage, RAW_EMAIL_PAYLOAD_FIELD};
use shared::redis_helper::publish_event;
use tokio::select;
use tracing::{error, info, warn};

use crate::config::EmailProcessorConfig;
use crate::repository::{EmailRepository, ParsedMessage};
use crate::storage::ObjectStorage;

pub struct EmailProcessor {
    config: EmailProcessorConfig,
    redis: ConnectionManager,
    repository: EmailRepository,
    object_storage: ObjectStorage,
}

impl EmailProcessor {
    pub fn new(
        config: EmailProcessorConfig,
        redis: ConnectionManager,
        repository: EmailRepository,
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

        let raw_email = raw_message.email;
        let raw_size_bytes = raw_email.data.len();
        let mailbox = raw_email
            .envelope
            .rcpt_to
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("raw email has no recipient"))?;
        let Some(account_id) = self.repository.find_account_id_by_address(&mailbox).await? else {
            warn!(stream_id = %entry.id, %mailbox, "unknown recipient, dropping email");
            return Ok(());
        };

        let parsed = build_parsed_message(
            raw_email,
            account_id,
            raw_size_bytes,
            self.config.mailbox.ttl_seconds,
            &self.object_storage,
        )
        .await?;
        let stored = self.repository.store_message(parsed).await?;

        let event = DomainEvent::EmailReceived(EmailReceivedEvent {
            account_id: stored.account_id,
            message_id: stored.message_id,
            mailbox: stored.mailbox.clone(),
            subject: stored.subject.clone(),
            from: stored.from.clone(),
            received_at: stored.received_at,
        });
        publish_event(&mut self.redis, EMAIL_RECEIVED_CHANNEL, &event).await?;

        info!(
            stream_id = %entry.id,
            account_id = %stored.account_id,
            message_id = %stored.message_id,
            mailbox = %stored.mailbox,
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

async fn build_parsed_message(
    raw: RawEmail,
    account_id: uuid::Uuid,
    raw_size_bytes: usize,
    _ttl_seconds: u64,
    object_storage: &ObjectStorage,
) -> Result<ParsedMessage> {
    let parsed = MessageParser::default()
        .parse(&raw.data)
        .ok_or_else(|| anyhow!("failed to parse raw email"))?;

    let id = new_uuid_v7();
    let mailbox = raw
        .envelope
        .rcpt_to
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("raw email has no recipient"))?;
    let from = parsed.from().and_then(first_addr);
    let to_addresses = parsed
        .to()
        .map(addresses_to_json)
        .unwrap_or_else(|| envelope_addresses_to_json(&raw.envelope.rcpt_to));
    let cc_addresses = parsed.cc().map(addresses_to_json);
    let bcc_addresses = parsed.bcc().map(addresses_to_json);
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
            .put_attachment(id, filename, content_type, part.contents().to_vec())
            .await?
        {
            Some(attachment) => attachments.push(attachment),
            None => warn!(message_id = %id, "attachment skipped because R2 is not configured"),
        }
    }

    Ok(ParsedMessage {
        id,
        account_id,
        mailbox,
        message_id: parsed.message_id().map(ToOwned::to_owned),
        in_reply_to: parsed.in_reply_to().as_text().map(ToOwned::to_owned),
        from_address: from
            .as_ref()
            .map(|addr| addr.address.clone())
            .unwrap_or_else(|| "unknown@unknown".to_owned()),
        from_name: from.and_then(|addr| addr.name),
        to_addresses,
        cc_addresses,
        bcc_addresses,
        subject: parsed.subject().map(ToOwned::to_owned),
        text_body,
        html_body,
        has_attachments: !attachments.is_empty(),
        size_bytes: raw_size_bytes as i32,
        received_at: raw.received_at,
        attachments,
    })
}

fn content_type_to_string(content_type: &mail_parser::ContentType<'_>) -> String {
    match content_type.subtype() {
        Some(subtype) => format!("{}/{}", content_type.ctype(), subtype),
        None => content_type.ctype().to_owned(),
    }
}

fn first_addr(address: &Address<'_>) -> Option<OwnedAddress> {
    address.iter().find_map(owned_addr)
}

fn addresses_to_json(addresses: &Address<'_>) -> serde_json::Value {
    serde_json::Value::Array(addresses.iter().filter_map(addr_to_json).collect())
}

fn envelope_addresses_to_json(addresses: &[String]) -> serde_json::Value {
    serde_json::json!(addresses
        .iter()
        .map(|address| serde_json::json!({ "address": address, "name": null }))
        .collect::<Vec<_>>())
}

fn addr_to_json(addr: &Addr<'_>) -> Option<serde_json::Value> {
    addr.address()
        .map(|address| serde_json::json!({ "address": address, "name": addr.name() }))
}

fn owned_addr(addr: &Addr<'_>) -> Option<OwnedAddress> {
    addr.address().map(|address| OwnedAddress {
        address: address.to_owned(),
        name: addr.name().map(ToOwned::to_owned),
    })
}

struct OwnedAddress {
    address: String,
    name: Option<String>,
}

struct StreamEntry {
    id: String,
    payload: String,
}
