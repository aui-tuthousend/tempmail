use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::models::{Account, EmailMessage, Mailbox, StoredMessage};
use uuid::Uuid;

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

#[derive(Debug, Deserialize)]
pub struct CreateAccountRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AccountAvailabilityQuery {
    pub local_part: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AccountAvailabilityResponse {
    pub local_part_available: Option<bool>,
    pub username_available: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub id: Uuid,
    pub username: String,
    pub address: String,
    pub display_name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Account> for AccountResponse {
    fn from(account: Account) -> Self {
        Self {
            id: account.id,
            username: account.username,
            address: account.address,
            display_name: account.display_name,
            is_active: account.is_active,
            created_at: account.created_at,
            updated_at: account.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username_or_email: String,
    pub password: String,
    pub device_info: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub session_id: Uuid,
    pub active_account_id: Uuid,
    pub expires_at: DateTime<Utc>,
    pub accounts: Vec<SessionAccountResponse>,
}

#[derive(Debug, Serialize)]
pub struct SessionAccountResponse {
    pub account_id: Uuid,
    pub username: String,
    pub address: String,
    pub display_name: Option<String>,
    pub is_active: bool,
    pub logged_in_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AccountIdPath {
    pub account_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct MessageIdPath {
    pub message_id: Uuid,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageView {
    Inbox,
    Starred,
    Archived,
    Deleted,
}

#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    pub view: Option<MessageView>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMessageRequest {
    pub is_read: Option<bool>,
    pub is_starred: Option<bool>,
    pub is_archived: Option<bool>,
    pub is_deleted: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub id: Uuid,
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

impl From<StoredMessage> for MessageResponse {
    fn from(message: StoredMessage) -> Self {
        Self {
            id: message.id,
            message_id: message.message_id,
            in_reply_to: message.in_reply_to,
            from_address: message.from_address,
            from_name: message.from_name,
            to_addresses: message.to_addresses,
            cc_addresses: message.cc_addresses,
            bcc_addresses: message.bcc_addresses,
            subject: message.subject,
            text_body: message.text_body,
            html_body: message.html_body,
            is_read: message.is_read,
            is_starred: message.is_starred,
            is_archived: message.is_archived,
            is_deleted: message.is_deleted,
            has_attachments: message.has_attachments,
            size_bytes: message.size_bytes,
            received_at: message.received_at,
            created_at: message.created_at,
            updated_at: message.updated_at,
        }
    }
}
