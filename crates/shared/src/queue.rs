use serde::{Deserialize, Serialize};

use uuid::Uuid;

use crate::models::RawEmail;

pub const RAW_EMAIL_PAYLOAD_FIELD: &str = "payload";
pub const SEND_EMAIL_PAYLOAD_FIELD: &str = "payload";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QueueMessage {
    RawEmail(RawEmailQueueMessage),
    SendEmail(SendEmailQueueMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawEmailQueueMessage {
    pub email: RawEmail,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SendEmailQueueMessage {
    pub outbox_id: Uuid,
    pub account_id: Uuid,
    pub from_address: String,
    pub to_addresses: Vec<String>,
    pub cc_addresses: Vec<String>,
    pub bcc_addresses: Vec<String>,
    pub subject: Option<String>,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub in_reply_to: Option<String>,
    pub attachment_keys: Vec<String>,
}

impl From<RawEmail> for QueueMessage {
    fn from(email: RawEmail) -> Self {
        Self::RawEmail(RawEmailQueueMessage { email })
    }
}
