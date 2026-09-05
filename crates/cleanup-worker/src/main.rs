use anyhow::Result;
use cleanup_worker::config::CleanupConfig;
use cleanup_worker::object_storage::ObjectStorage;
use cleanup_worker::worker::CleanupWorker;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = CleanupConfig::from_env()?;
    let db = PgPoolOptions::new()
        .max_connections(2)
        .connect(&config.postgres.database_url)
        .await?;
    let object_storage = ObjectStorage::from_config(&config.r2).await;

    CleanupWorker::new(config, db, object_storage)
        .run_until_shutdown()
        .await
}
