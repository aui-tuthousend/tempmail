use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;
use shared::models::Attachment;
use uuid::Uuid;

use crate::config::R2Config;

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

    pub async fn put_attachment(
        &self,
        message_id: Uuid,
        filename: Option<String>,
        content_type: String,
        bytes: Vec<u8>,
    ) -> Result<Option<Attachment>> {
        let Some(client) = &self.client else {
            return Ok(None);
        };

        let attachment_id = Uuid::new_v4();
        let storage_key = format!("{}{message_id}/{attachment_id}", self.prefix);
        let size_bytes = bytes.len() as u64;

        client
            .put_object()
            .bucket(&self.bucket)
            .key(&storage_key)
            .content_type(&content_type)
            .body(ByteStream::from(bytes))
            .send()
            .await?;

        Ok(Some(Attachment {
            id: attachment_id,
            filename,
            content_type,
            size_bytes,
            storage_key,
        }))
    }
}
