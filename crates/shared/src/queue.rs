use serde::{Deserialize, Serialize};

use crate::models::RawEmail;

pub const RAW_EMAIL_PAYLOAD_FIELD: &str = "payload";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum QueueMessage {
    RawEmail(RawEmailQueueMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawEmailQueueMessage {
    pub email: RawEmail,
}

impl From<RawEmail> for QueueMessage {
    fn from(email: RawEmail) -> Self {
        Self::RawEmail(RawEmailQueueMessage { email })
    }
}
