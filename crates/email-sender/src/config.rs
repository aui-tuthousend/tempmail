use shared::config::{
    env_or, env_parse_or, load_dotenv, PostgresConfig, QueueConfig, RedisConfig, SmtpRelayConfig,
};
use shared::Result;

#[derive(Debug, Clone)]
pub struct EmailSenderConfig {
    pub redis: RedisConfig,
    pub postgres: PostgresConfig,
    pub queue: QueueConfig,
    pub smtp: SmtpRelayConfig,
    pub r2: R2Config,
    pub consumer_group: String,
    pub consumer_name: String,
    pub batch_size: usize,
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

impl EmailSenderConfig {
    pub fn from_env() -> Result<Self> {
        load_dotenv();

        Ok(Self {
            redis: RedisConfig::from_env()?,
            postgres: PostgresConfig::from_env()?,
            queue: QueueConfig::from_env()?,
            smtp: SmtpRelayConfig::from_env()?,
            r2: R2Config::from_env(),
            consumer_group: env_or("EMAIL_SENDER_CONSUMER_GROUP", "email-sender"),
            consumer_name: env_or("EMAIL_SENDER_CONSUMER_NAME", "email-sender-1"),
            batch_size: env_parse_or("EMAIL_SENDER_BATCH_SIZE", 5)?,
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
