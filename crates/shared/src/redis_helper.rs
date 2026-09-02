use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::events::DomainEvent;
use crate::queue::{QueueMessage, RAW_EMAIL_PAYLOAD_FIELD};
use crate::Result;

pub async fn connection_manager(redis_url: &str) -> Result<ConnectionManager> {
    let client = redis::Client::open(redis_url)?;
    Ok(client.get_connection_manager().await?)
}

pub async fn push_queue_message(
    conn: &mut ConnectionManager,
    stream: &str,
    message: &QueueMessage,
) -> Result<String> {
    let payload = serde_json::to_string(message)?;
    let id = redis::cmd("XADD")
        .arg(stream)
        .arg("*")
        .arg(RAW_EMAIL_PAYLOAD_FIELD)
        .arg(payload)
        .query_async(conn)
        .await?;

    Ok(id)
}

pub async fn publish_event(
    conn: &mut ConnectionManager,
    channel: &str,
    event: &DomainEvent,
) -> Result<u64> {
    let payload = serde_json::to_string(event)?;
    let subscriber_count = redis::cmd("PUBLISH")
        .arg(channel)
        .arg(payload)
        .query_async(conn)
        .await?;

    Ok(subscriber_count)
}

pub async fn set_json<T>(
    conn: &mut ConnectionManager,
    key: &str,
    value: &T,
    ttl_seconds: u64,
) -> Result<()>
where
    T: Serialize,
{
    let payload = serde_json::to_string(value)?;
    let _: () = conn.set_ex(key, payload, ttl_seconds).await?;
    Ok(())
}

pub async fn get_json<T>(conn: &mut ConnectionManager, key: &str) -> Result<Option<T>>
where
    T: DeserializeOwned,
{
    let payload: Option<String> = conn.get(key).await?;
    payload
        .map(|value| serde_json::from_str(&value))
        .transpose()
        .map_err(Into::into)
}

pub async fn delete_key(conn: &mut ConnectionManager, key: &str) -> Result<u64> {
    Ok(conn.del(key).await?)
}
