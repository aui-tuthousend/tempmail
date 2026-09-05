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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {
    pub id: Uuid,
    pub username: String,
    pub local_part: String,
    pub domain: String,
    pub address: String,
    pub display_name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountCredentials {
    pub id: Uuid,
    pub username: String,
    pub address: String,
    pub password_hash: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Session {
    pub id: Uuid,
    pub token_hash: String,
    pub device_info: Option<serde_json::Value>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionAccount {
    pub id: Uuid,
    pub session_id: Uuid,
    pub account_id: Uuid,
    pub is_active: bool,
    pub logged_in_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiKey {
    pub id: Uuid,
    pub name: Option<String>,
    pub key_hash: String,
    pub key_prefix: String,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoredMessage {
    pub id: Uuid,
    pub account_id: Uuid,
    pub message_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub from_address: String,
    pub from_name: Option<String>,
    pub to_addresses: serde_json::Value,
    pub cc_addresses: Option<serde_json::Value>,
    pub bcc_addresses: Option<serde_json::Value>,
    pub subject: Option<String>,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub is_read: bool,
    pub is_starred: bool,
    pub is_archived: bool,
    pub is_deleted: bool,
    pub has_attachments: bool,
    pub size_bytes: i32,
    pub received_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoredAttachment {
    pub id: Uuid,
    pub message_id: Uuid,
    pub filename: String,
    pub content_type: Option<String>,
    pub size_bytes: i32,
    pub storage_key: String,
    pub content_id: Option<String>,
    pub created_at: DateTime<Utc>,
}
