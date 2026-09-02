use std::time::Duration;

use shared::config::{env_parse_or, load_dotenv, RedisConfig};
use shared::Result;

#[derive(Debug, Clone)]
pub struct CleanupConfig {
    pub redis: RedisConfig,
    pub interval: Duration,
    pub batch_size: usize,
    pub r2: R2Config,
}

#[derive(Debug, Clone)]
pub struct R2Config {
    pub endpoint: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub region: String,
}

impl CleanupConfig {
    pub fn from_env() -> Result<Self> {
        load_dotenv();

        Ok(Self {
            redis: RedisConfig::from_env()?,
            interval: Duration::from_secs(env_parse_or("CLEANUP_INTERVAL_SECONDS", 60)?),
            batch_size: env_parse_or("CLEANUP_BATCH_SIZE", 100)?,
            r2: R2Config::from_env(),
        })
    }
}

impl R2Config {
    pub fn from_env() -> Self {
        Self {
            endpoint: shared::config::env_or("R2_ENDPOINT", ""),
            bucket: shared::config::env_or("R2_BUCKET", ""),
            access_key_id: shared::config::env_or("R2_ACCESS_KEY_ID", ""),
            secret_access_key: shared::config::env_or("R2_SECRET_ACCESS_KEY", ""),
            region: shared::config::env_or("R2_REGION", "auto"),
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.endpoint.is_empty()
            && !self.bucket.is_empty()
            && !self.access_key_id.is_empty()
            && !self.secret_access_key.is_empty()
    }
}
