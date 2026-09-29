use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::Client;
use uuid::Uuid;

use crate::config::R2Config;

#[derive(Clone)]
pub struct ObjectStorage {
    client: Option<Client>,
    bucket: String,
}

impl ObjectStorage {
    pub async fn from_config(config: &R2Config) -> Self {
        if !config.is_configured() {
            return Self {
                client: None,
                bucket: String::new(),
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
        }
    }

    pub async fn get_attachment_bytes(&self, storage_key: &str) -> Result<Option<Vec<u8>>> {
        let Some(client) = &self.client else {
            return Ok(None);
        };

        let output = client
            .get_object()
            .bucket(&self.bucket)
            .key(storage_key)
            .send()
            .await?;

        let bytes = output.body.collect().await?.into_bytes().to_vec();
        Ok(Some(bytes))
    }

    pub async fn get_attachment_for_message(
        &self,
        message_id: Uuid,
        attachment_id: Uuid,
        prefix: &str,
    ) -> Result<Option<Vec<u8>>> {
        let storage_key = format!("{}{message_id}/{attachment_id}", prefix);
        self.get_attachment_bytes(&storage_key).await
    }
}
