use shared::config::{env_or, env_parse_or, load_dotenv, MailboxConfig, RedisConfig};
use shared::Result;

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub bind_addr: String,
    pub allowed_origins: String,
    pub mailbox_local_part_length: usize,
    pub redis: RedisConfig,
    pub mailbox: MailboxConfig,
}

impl ApiConfig {
    pub fn from_env() -> Result<Self> {
        load_dotenv();

        Ok(Self {
            bind_addr: env_or("API_BIND_ADDR", "0.0.0.0:8080"),
            allowed_origins: env_or("CORS_ALLOWED_ORIGINS", "*"),
            mailbox_local_part_length: env_parse_or("MAILBOX_LOCAL_PART_LENGTH", 12)?,
            redis: RedisConfig::from_env()?,
            mailbox: MailboxConfig::from_env()?,
        })
    }
}
