use anyhow::Result;
use api_server::config::ApiConfig;
use api_server::routes::router;
use api_server::state::AppState;
use axum::http::{HeaderValue, Method};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = ApiConfig::from_env()?;
    let redis = shared::redis_helper::connection_manager(&config.redis.url).await?;
    let bind_addr = config.bind_addr.clone();
    let cors = cors_layer(&config.allowed_origins)?;
    let app = router(AppState::new(config, redis), cors);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    info!(%bind_addr, "api server listening");
    axum::serve(listener, app).await?;
    Ok(())
}

fn cors_layer(allowed_origins: &str) -> Result<CorsLayer> {
    let layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    if allowed_origins == "*" {
        return Ok(layer.allow_origin(Any));
    }

    let origins = allowed_origins
        .split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .map(HeaderValue::from_str)
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(layer.allow_origin(origins))
}
