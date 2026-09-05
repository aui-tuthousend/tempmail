use shared::config::{
    env_or, env_parse_or, load_dotenv, MailboxConfig, PostgresConfig, RedisConfig,
};
use shared::Result;

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub bind_addr: String,
    pub allowed_origins: String,
    pub mailbox_local_part_length: usize,
    pub session_cookie_name: String,
    pub session_ttl_seconds: i64,
    pub session_cookie_secure: bool,
    pub redis: RedisConfig,
    pub postgres: PostgresConfig,
    pub mailbox: MailboxConfig,
}

impl ApiConfig {
    pub fn from_env() -> Result<Self> {
        load_dotenv();

        Ok(Self {
            bind_addr: env_or("API_BIND_ADDR", "0.0.0.0:8080"),
            allowed_origins: env_or("CORS_ALLOWED_ORIGINS", "*"),
            mailbox_local_part_length: env_parse_or("MAILBOX_LOCAL_PART_LENGTH", 12)?,
            session_cookie_name: env_or("SESSION_COOKIE_NAME", "tempmail_session"),
            session_ttl_seconds: env_parse_or("SESSION_TTL_SECONDS", 2_592_000)?,
            session_cookie_secure: env_parse_or("SESSION_COOKIE_SECURE", false)?,
            redis: RedisConfig::from_env()?,
            postgres: PostgresConfig::from_env()?,
            mailbox: MailboxConfig::from_env()?,
        })
    }
}
