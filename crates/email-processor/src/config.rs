use shared::config::{env_or, env_parse_or, load_dotenv, MailboxConfig, QueueConfig, RedisConfig};
use shared::Result;

#[derive(Debug, Clone)]
pub struct EmailProcessorConfig {
    pub redis: RedisConfig,
    pub queue: QueueConfig,
    pub mailbox: MailboxConfig,
    pub consumer_name: String,
    pub batch_size: usize,
    pub message_ttl_grace_seconds: u64,
    pub r2: R2Config,
}

#[derive(Debug, Clone)]
pub struct R2Config {
    pub endpoint: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub region: String,
    pub prefix: String,
}

impl EmailProcessorConfig {
    pub fn from_env() -> Result<Self> {
        load_dotenv();

        Ok(Self {
            redis: RedisConfig::from_env()?,
            queue: QueueConfig::from_env()?,
            mailbox: MailboxConfig::from_env()?,
            consumer_name: env_or("REDIS_CONSUMER_NAME", "email-processor-1"),
            batch_size: env_parse_or("EMAIL_PROCESSOR_BATCH_SIZE", 10)?,
            message_ttl_grace_seconds: env_parse_or("MESSAGE_TTL_GRACE_SECONDS", 300)?,
            r2: R2Config::from_env(),
        })
    }
}

impl R2Config {
    pub fn from_env() -> Self {
        Self {
            endpoint: env_or("R2_ENDPOINT", ""),
            bucket: env_or("R2_BUCKET", ""),
            access_key_id: env_or("R2_ACCESS_KEY_ID", ""),
            secret_access_key: env_or("R2_SECRET_ACCESS_KEY", ""),
            region: env_or("R2_REGION", "auto"),
            prefix: env_or("R2_PREFIX", "attachments/"),
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.endpoint.is_empty()
            && !self.bucket.is_empty()
            && !self.access_key_id.is_empty()
            && !self.secret_access_key.is_empty()
    }
}
