use anyhow::Result;
use email_processor::config::EmailProcessorConfig;
use email_processor::processor::EmailProcessor;
use email_processor::repository::EmailRepository;
use email_processor::storage::ObjectStorage;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = EmailProcessorConfig::from_env()?;
    let redis = shared::redis_helper::connection_manager(&config.redis.url).await?;
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.postgres.database_url)
        .await?;
    let repository = EmailRepository::new(db);
    let object_storage = ObjectStorage::from_config(&config.r2).await;

    EmailProcessor::new(config, redis, repository, object_storage)
        .run_until_shutdown()
        .await
}
