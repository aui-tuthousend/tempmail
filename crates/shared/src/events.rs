use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const EMAIL_RECEIVED_CHANNEL: &str = "email.received";
pub const EMAIL_SENT_CHANNEL: &str = "email.sent";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DomainEvent {
    EmailReceived(EmailReceivedEvent),
    EmailSent(EmailSentEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmailReceivedEvent {
    pub account_id: Uuid,
    pub message_id: Uuid,
    pub mailbox: String,
    pub subject: Option<String>,
    pub from: Option<String>,
    pub received_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmailSentEvent {
    pub account_id: Uuid,
    pub outbox_id: Uuid,
    pub status: String,
}
