use std::env;
use std::str::FromStr;

use crate::{Result, TempMailError};

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct QueueConfig {
    pub raw_email_stream: String,
    pub consumer_group: String,
    pub stream_block_ms: u64,
}

#[derive(Debug, Clone)]
pub struct MailboxConfig {
    pub domain: String,
    pub ttl_seconds: u64,
}

#[derive(Debug, Clone)]
pub struct SharedConfig {
    pub redis: RedisConfig,
    pub queue: QueueConfig,
    pub mailbox: MailboxConfig,
}

impl SharedConfig {
    pub fn from_env() -> Result<Self> {
        load_dotenv();

        Ok(Self {
            redis: RedisConfig::from_env()?,
            queue: QueueConfig::from_env()?,
            mailbox: MailboxConfig::from_env()?,
        })
    }
}

impl RedisConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            url: env_or("REDIS_URL", "redis://127.0.0.1:6379"),
        })
    }
}

impl QueueConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            raw_email_stream: env_or("QUEUE_RAW_EMAIL_STREAM", "email_raw"),
            consumer_group: env_or("REDIS_CONSUMER_GROUP", "email-processor"),
            stream_block_ms: env_parse_or("REDIS_STREAM_BLOCK_MS", 5_000)?,
        })
    }
}

impl MailboxConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            domain: env_required("MAILBOX_DOMAIN")?,
            ttl_seconds: env_parse_or("MAILBOX_TTL_SECONDS", 3_600)?,
        })
    }
}

pub fn load_dotenv() {
    let _ = dotenvy::dotenv();
}

pub fn env_required(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| TempMailError::MissingEnv(name))
}

pub fn env_or(name: &'static str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_owned())
}

pub fn env_parse_or<T>(name: &'static str, default: T) -> Result<T>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    match env::var(name) {
        Ok(value) => value
            .parse::<T>()
            .map_err(|source| TempMailError::InvalidEnv {
                name,
                message: source.to_string(),
            }),
        Err(_) => Ok(default),
    }
}
