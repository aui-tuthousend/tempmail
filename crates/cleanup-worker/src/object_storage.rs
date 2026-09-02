use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::Client;

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

    pub async fn delete_object(&self, key: &str) -> Result<()> {
        let Some(client) = &self.client else {
            return Ok(());
        };

        client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;

        Ok(())
    }
}
