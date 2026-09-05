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

    #[error("auth configuration error: {message}")]
    AuthConfig { message: String },

    #[error("password hash error: {message}")]
    PasswordHash { message: String },

    #[error("authentication failed")]
    AuthenticationFailed,

    #[error("authorization failed")]
    AuthorizationFailed,

    #[error("session expired")]
    SessionExpired,

    #[error("invalid session")]
    InvalidSession,

    #[error("api key is required")]
    ApiKeyRequired,

    #[error("invalid api key")]
    InvalidApiKey,

    #[error("rate limit exceeded")]
    RateLimitExceeded,

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("resource not found: {0}")]
    NotFound(String),
}
