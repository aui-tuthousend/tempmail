use thiserror::Error;

pub type Result<T> = std::result::Result<T, TempMailError>;

#[derive(Debug, Error)]
pub enum TempMailError {
    #[error("missing environment variable: {0}")]
    MissingEnv(&'static str),

    #[error("invalid environment variable {name}: {message}")]
    InvalidEnv { name: &'static str, message: String },

    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("invalid email address: {0}")]
    InvalidEmailAddress(String),

    #[error("resource not found: {0}")]
    NotFound(String),
}
