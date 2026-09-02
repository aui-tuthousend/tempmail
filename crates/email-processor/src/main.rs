use anyhow::Result;
use email_processor::config::EmailProcessorConfig;
use email_processor::processor::EmailProcessor;
use email_processor::storage::ObjectStorage;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = EmailProcessorConfig::from_env()?;
    let redis = shared::redis_helper::connection_manager(&config.redis.url).await?;
    let object_storage = ObjectStorage::from_config(&config.r2).await;

    EmailProcessor::new(config, redis, object_storage)
        .run_until_shutdown()
        .await
}
