use shared::config::{env_or, env_parse_or, load_dotenv, QueueConfig, RedisConfig};
use shared::Result;

#[derive(Debug, Clone)]
pub struct SmtpReceiverConfig {
    pub listen_addr: String,
    pub hostname: String,
    pub mailbox_domain: String,
    pub max_message_bytes: usize,
    pub max_messages_per_connection: usize,
    pub rate_limit_window_seconds: i64,
    pub rate_limit_messages_per_ip: usize,
    pub rate_limit_messages_per_sender_domain: usize,
    pub enable_starttls: bool,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
    pub redis: RedisConfig,
    pub queue: QueueConfig,
}

impl SmtpReceiverConfig {
    pub fn from_env() -> Result<Self> {
        load_dotenv();

        Ok(Self {
            listen_addr: env_or("SMTP_LISTEN_ADDR", "0.0.0.0:2525"),
            hostname: env_or("SMTP_HOSTNAME", "localhost"),
            mailbox_domain: env_or("MAILBOX_DOMAIN", "localhost"),
            max_message_bytes: env_parse_or("SMTP_MAX_MESSAGE_BYTES", 10 * 1024 * 1024)?,
            max_messages_per_connection: env_parse_or("SMTP_MAX_MESSAGES_PER_CONNECTION", 20)?,
            rate_limit_window_seconds: env_parse_or("SMTP_RATE_LIMIT_WINDOW_SECONDS", 60)?,
            rate_limit_messages_per_ip: env_parse_or("SMTP_RATE_LIMIT_MESSAGES_PER_IP", 30)?,
            rate_limit_messages_per_sender_domain: env_parse_or(
                "SMTP_RATE_LIMIT_MESSAGES_PER_SENDER_DOMAIN",
                100,
            )?,
            enable_starttls: env_parse_or("SMTP_ENABLE_STARTTLS", false)?,
            tls_cert_path: optional_env("TLS_CERT_PATH"),
            tls_key_path: optional_env("TLS_KEY_PATH"),
            redis: RedisConfig::from_env()?,
            queue: QueueConfig::from_env()?,
        })
    }

    pub fn starttls_configured(&self) -> bool {
        self.enable_starttls && self.tls_cert_path.is_some() && self.tls_key_path.is_some()
    }
}

fn optional_env(name: &'static str) -> Option<String> {
    let value = env_or(name, "");
    (!value.is_empty()).then_some(value)
}
