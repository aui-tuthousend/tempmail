use anyhow::Result;
use cleanup_worker::config::CleanupConfig;
use cleanup_worker::object_storage::ObjectStorage;
use cleanup_worker::worker::CleanupWorker;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = CleanupConfig::from_env()?;
    let redis = shared::redis_helper::connection_manager(&config.redis.url).await?;
    let object_storage = ObjectStorage::from_config(&config.r2).await;

    CleanupWorker::new(config, redis, object_storage)
        .run_until_shutdown()
        .await
}
