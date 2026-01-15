use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    Client,
    config::{Credentials, Region},
    primitives::ByteStream,
};
use std::sync::Arc;
use tokio::sync::OnceCell;

static R2_CLIENT: OnceCell<Arc<R2Client>> = OnceCell::const_new();

pub struct R2Client {
    client: Client,
    bucket: String,
}

impl R2Client {
    pub async fn new() -> Result<Self, String> {
        let account_id =
            std::env::var("R2_ACCOUNT_ID").map_err(|_| "R2_ACCOUNT_ID not set".to_string())?;
        let access_key_id = std::env::var("R2_ACCESS_KEY_ID")
            .map_err(|_| "R2_ACCESS_KEY_ID not set".to_string())?;
        let secret_access_key = std::env::var("R2_SECRET_ACCESS_KEY")
            .map_err(|_| "R2_SECRET_ACCESS_KEY not set".to_string())?;
        let bucket = std::env::var("R2_BUCKET").map_err(|_| "R2_BUCKET not set".to_string())?;

        let endpoint = format!("https://{account_id}.r2.cloudflarestorage.com");

        let credentials_provider =
            Credentials::new(access_key_id, secret_access_key, None, None, "static");

        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new("auto"))
            .endpoint_url(endpoint)
            .credentials_provider(credentials_provider)
            .load()
            .await;

        let client = Client::new(&config);

        Ok(Self { client, bucket })
    }

    pub async fn get() -> Option<Arc<R2Client>> {
        R2_CLIENT
            .get_or_try_init(|| async {
                match R2Client::new().await {
                    Ok(client) => Ok(Arc::new(client)),
                    Err(e) => {
                        tracing::warn!(error = %e, "Failed to initialize R2 client, OG images will not be cached");
                        Err(e)
                    }
                }
            })
            .await
            .ok()
            .cloned()
    }

    pub async fn get_object(&self, key: &str) -> Result<Vec<u8>, String> {
        let result = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| format!("Failed to get object from R2: {e}"))?;

        let bytes = result
            .body
            .collect()
            .await
            .map_err(|e| format!("Failed to read object body: {e}"))?
            .into_bytes()
            .to_vec();

        Ok(bytes)
    }

    pub async fn put_object(
        &self,
        key: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> Result<(), String> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(body))
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| format!("Failed to put object to R2: {e}"))?;

        Ok(())
    }

    pub async fn delete_object(&self, key: &str) -> Result<(), String> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| format!("Failed to delete object from R2: {e}"))?;

        Ok(())
    }

    /// Delete all objects under a prefix (e.g., "projects/{id}/{ulid}/")
    pub async fn delete_prefix(&self, prefix: &str) -> Result<usize, String> {
        let list_result = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(prefix)
            .send()
            .await
            .map_err(|e| format!("Failed to list objects in R2: {e}"))?;

        let mut deleted = 0;
        if let Some(contents) = list_result.contents {
            for object in contents {
                if let Some(key) = object.key {
                    self.delete_object(&key).await?;
                    deleted += 1;
                }
            }
        }

        Ok(deleted)
    }

    pub async fn object_exists(&self, key: &str) -> bool {
        self.client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .is_ok()
    }
}
