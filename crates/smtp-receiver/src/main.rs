use anyhow::Result;
use smtp_receiver::config::SmtpReceiverConfig;
use smtp_receiver::server::{run, AppState};
use smtp_receiver::tls::tls_acceptor;
use tracing::warn;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = SmtpReceiverConfig::from_env()?;
    let redis = shared::redis_helper::connection_manager(&config.redis.url).await?;

    let tls_acceptor = if config.starttls_configured() {
        Some(tls_acceptor(
            config
                .tls_cert_path
                .as_deref()
                .expect("TLS cert path checked"),
            config
                .tls_key_path
                .as_deref()
                .expect("TLS key path checked"),
        )?)
    } else {
        if config.enable_starttls {
            warn!("SMTP_ENABLE_STARTTLS=true but TLS_CERT_PATH/TLS_KEY_PATH is missing; STARTTLS disabled");
        }
        None
    };

    run(AppState::new(config, redis, tls_acceptor)).await
}
