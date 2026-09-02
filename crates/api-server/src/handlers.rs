use std::convert::Infallible;
use std::time::Duration;

use async_stream::stream;
use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use chrono::{Duration as ChronoDuration, Utc};
use futures_util::Stream;
use futures_util::StreamExt;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use shared::events::{DomainEvent, EMAIL_RECEIVED_CHANNEL};
use shared::keys::{mailbox_index_key, mailbox_key, message_key};
use shared::models::{EmailAddress, EmailMessage, Mailbox};
use shared::redis_helper::{get_json, set_json};
use tracing::{error, warn};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct GenerateMailboxResponse {
    pub mailbox: Mailbox,
    pub address: String,
}

#[derive(Debug, Serialize)]
pub struct ListMessagesResponse {
    pub mailbox: String,
    pub messages: Vec<EmailMessage>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Deserialize)]
pub struct MailboxPath {
    pub mailbox: String,
}

pub async fn generate_mailbox(State(state): State<AppState>) -> Json<GenerateMailboxResponse> {
    let mailbox = new_mailbox(&state);
    let mut redis = state.redis.clone();
    let key = mailbox_key(&mailbox.address.as_string());

    if let Err(error) = set_json(&mut redis, &key, &mailbox, state.config.mailbox.ttl_seconds).await
    {
        error!(%error, "failed to store generated mailbox");
    }

    Json(GenerateMailboxResponse {
        address: mailbox.address.as_string(),
        mailbox,
    })
}

pub async fn list_messages(
    State(state): State<AppState>,
    Path(path): Path<MailboxPath>,
) -> Json<ListMessagesResponse> {
    let mailbox = path.mailbox;
    let mut redis = state.redis.clone();
    let message_ids = match mailbox_message_ids(&mut redis, &mailbox).await {
        Ok(message_ids) => message_ids,
        Err(error) => {
            error!(%error, %mailbox, "failed to load mailbox index");
            Vec::new()
        }
    };

    let mut messages = Vec::new();
    for message_id in message_ids {
        let key = message_key(&mailbox, message_id);
        match get_json::<EmailMessage>(&mut redis, &key).await {
            Ok(Some(message)) => messages.push(message),
            Ok(None) => {}
            Err(error) => warn!(%error, %key, "failed to load message"),
        }
    }

    Json(ListMessagesResponse { mailbox, messages })
}

pub async fn inbox_events(
    State(state): State<AppState>,
    Path(path): Path<MailboxPath>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mailbox = path.mailbox;
    let redis_url = state.config.redis.url.clone();

    let events = stream! {
        match redis::Client::open(redis_url) {
            Ok(client) => match client.get_async_pubsub().await {
                Ok(mut pubsub) => {
                    if let Err(error) = pubsub.subscribe(EMAIL_RECEIVED_CHANNEL).await {
                        error!(%error, "failed to subscribe to Redis Pub/Sub");
                        yield Ok(Event::default().event("error").data("subscription_failed"));
                        return;
                    }

                    let mut messages = pubsub.on_message();
                    while let Some(message) = messages.next().await {
                        let payload = match message.get_payload::<String>() {
                            Ok(payload) => payload,
                            Err(error) => {
                                warn!(%error, "failed to read Pub/Sub payload");
                                continue;
                            }
                        };

                        let event = match serde_json::from_str::<DomainEvent>(&payload) {
                            Ok(event) => event,
                            Err(error) => {
                                warn!(%error, "failed to decode domain event");
                                continue;
                            }
                        };

                        let DomainEvent::EmailReceived(email_received) = event;
                        if email_received.mailbox != mailbox {
                            continue;
                        }

                        yield Ok(Event::default()
                            .event("email.received")
                            .id(email_received.message_id.to_string())
                            .data(payload));
                    }
                }
                Err(error) => {
                    error!(%error, "failed to create Redis Pub/Sub connection");
                    yield Ok(Event::default().event("error").data("pubsub_connection_failed"));
                }
            },
            Err(error) => {
                error!(%error, "failed to create Redis client");
                yield Ok(Event::default().event("error").data("redis_client_failed"));
            }
        }
    };

    Sse::new(events).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

fn new_mailbox(state: &AppState) -> Mailbox {
    let local_part = Uuid::new_v4()
        .simple()
        .to_string()
        .chars()
        .take(state.config.mailbox_local_part_length)
        .collect::<String>();
    let address = EmailAddress::new(local_part, state.config.mailbox.domain.clone());
    let created_at = Utc::now();
    let expires_at = created_at + ChronoDuration::seconds(state.config.mailbox.ttl_seconds as i64);

    Mailbox {
        id: Uuid::new_v4(),
        address,
        created_at,
        expires_at,
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
