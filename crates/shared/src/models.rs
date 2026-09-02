use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmailAddress {
    pub local_part: String,
    pub domain: String,
}

impl EmailAddress {
    pub fn new(local_part: impl Into<String>, domain: impl Into<String>) -> Self {
        Self {
            local_part: local_part.into(),
            domain: domain.into(),
        }
    }

    pub fn as_string(&self) -> String {
        format!("{}@{}", self.local_part, self.domain)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Mailbox {
    pub id: Uuid,
    pub address: EmailAddress,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Envelope {
    pub mail_from: String,
    pub rcpt_to: Vec<String>,
    pub remote_addr: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawEmail {
    pub id: Uuid,
    pub envelope: Envelope,
    pub data: Vec<u8>,
    pub received_at: DateTime<Utc>,
}

impl RawEmail {
    pub fn new(envelope: Envelope, data: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4(),
            envelope,
            data,
            received_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Attachment {
    pub id: Uuid,
    pub filename: Option<String>,
    pub content_type: String,
    pub size_bytes: u64,
    pub storage_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmailMessage {
    pub id: Uuid,
    pub mailbox: String,
    pub from: Option<String>,
    pub to: Vec<String>,
    pub subject: Option<String>,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub attachments: Vec<Attachment>,
    pub received_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
