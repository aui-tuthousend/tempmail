use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const EMAIL_RECEIVED_CHANNEL: &str = "email.received";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DomainEvent {
    EmailReceived(EmailReceivedEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmailReceivedEvent {
    pub message_id: Uuid,
    pub mailbox: String,
    pub subject: Option<String>,
    pub from: Option<String>,
    pub received_at: DateTime<Utc>,
}
