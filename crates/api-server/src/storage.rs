use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use shared::ids::new_uuid_v7;
use uuid::Uuid;

use crate::config::R2Config;

#[derive(Debug, Clone)]
pub struct StoredUpload {
    pub id: Uuid,
    pub storage_key: String,
    pub size_bytes: usize,
}

#[derive(Clone)]
pub struct ObjectStorage {
    client: Option<Client>,
    bucket: String,
    prefix: String,
}

impl ObjectStorage {
    pub async fn from_config(config: &R2Config) -> Self {
        if !config.is_configured() {
            return Self {
                client: None,
                bucket: String::new(),
                prefix: String::new(),
            };
        }

        let credentials = Credentials::new(
            config.access_key_id.clone(),
            config.secret_access_key.clone(),
            None,
            None,
            "r2-env",
        );
        let sdk_config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(config.region.clone()))
            .endpoint_url(config.endpoint.clone())
            .credentials_provider(credentials)
            .load()
            .await;
        let s3_config = aws_sdk_s3::config::Builder::from(&sdk_config)
            .force_path_style(true)
            .build();

        Self {
            client: Some(Client::from_conf(s3_config)),
            bucket: config.bucket.clone(),
            prefix: config.prefix.clone(),
        }
    }

    pub async fn put_outbox_attachment(
        &self,
        outbox_id: Uuid,
        filename: &str,
        content_type: Option<&str>,
        bytes: Vec<u8>,
    ) -> Result<StoredUpload> {
        let attachment_id = new_uuid_v7();
        let storage_key = format!(
            "{}outbox/{outbox_id}/{attachment_id}/{filename}",
            self.prefix
        );
        let size_bytes = bytes.len();

        if let Some(client) = &self.client {
            let mut request = client
                .put_object()
                .bucket(&self.bucket)
                .key(&storage_key)
                .body(ByteStream::from(bytes));

            if let Some(content_type) = content_type {
                request = request.content_type(content_type);
            }

            request.send().await?;
        }

        Ok(StoredUpload {
            id: attachment_id,
            storage_key,
            size_bytes,
        })
    }
}
