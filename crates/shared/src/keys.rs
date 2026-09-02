use uuid::Uuid;

pub fn mailbox_key(mailbox: &str) -> String {
    format!("mailbox:{mailbox}")
}

pub fn mailbox_index_key(mailbox: &str) -> String {
    format!("mailbox:{mailbox}:messages")
}

pub fn message_key(mailbox: &str, message_id: Uuid) -> String {
    format!("mailbox:{mailbox}:message:{message_id}")
}

pub fn message_expiry_index_key() -> &'static str {
    "messages:expires_at"
}
