use anyhow::Result;
use api_server::config::ApiConfig;
use api_server::repositories::{
    AccountRepository, ApiKeyRepository, MailboxRepository, MessageRepository, SessionRepository,
};
use api_server::routes::router;
use api_server::services::{
    AccountService, EventService, MailboxService, MessageService, SessionService,
};
use api_server::state::{AppServices, AppState};
use axum::http::{HeaderValue, Method};
use shared::auth::{PasswordService, TokenService};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = ApiConfig::from_env()?;
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.postgres.database_url)
        .await?;
    let redis = shared::redis_helper::connection_manager(&config.redis.url).await?;

    let password_service = PasswordService::new()?;
    let token_service = TokenService::new();

    let mailbox_repository = MailboxRepository::new(redis.clone());
    let mailbox_service = MailboxService::new(
        mailbox_repository,
        config.mailbox.domain.clone(),
        config.mailbox_local_part_length,
        config.mailbox.ttl_seconds,
    );
    let event_service = EventService::new(config.redis.url.clone());

    let account_repository = AccountRepository::new(db.clone());
    let api_key_repository = ApiKeyRepository::new(db.clone());
    let account_service = AccountService::new(
        account_repository,
        api_key_repository,
        password_service.clone(),
        token_service.clone(),
    );

    let session_repository = SessionRepository::new(db.clone());
    let session_service = SessionService::new(
        session_repository,
        password_service,
        token_service,
        config.session_ttl_seconds,
    );

    let message_repository = MessageRepository::new(db.clone());
    let message_service = MessageService::new(message_repository, session_service.clone());

    let bind_addr = config.bind_addr.clone();
    let cors = cors_layer(&config.allowed_origins)?;
    let services = AppServices {
        mailbox_service,
        event_service,
        account_service,
        session_service,
        message_service,
    };
    let app = router(
        AppState::new(config, db, redis.clone(), redis, services),
        cors,
    );
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    info!(%bind_addr, "api server listening");
    axum::serve(listener, app).await?;
    Ok(())
}

fn cors_layer(allowed_origins: &str) -> Result<CorsLayer> {
    let layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::OPTIONS])
        .allow_headers(AllowHeaders::mirror_request())
        .allow_credentials(true);

    let origins = if allowed_origins == "*" {
        vec![
            HeaderValue::from_static("http://localhost:3000"),
            HeaderValue::from_static("http://localhost:5173"),
            HeaderValue::from_static("http://127.0.0.1:3000"),
            HeaderValue::from_static("http://127.0.0.1:5173"),
        ]
    } else {
        allowed_origins
            .split(',')
            .map(str::trim)
            .filter(|origin| !origin.is_empty())
            .map(HeaderValue::from_str)
            .collect::<std::result::Result<Vec<_>, _>>()?
    };

    Ok(layer.allow_origin(AllowOrigin::list(origins)))
}
