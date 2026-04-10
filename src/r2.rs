use opendal::{Operator, services::S3};
use std::sync::Arc;
use tokio::sync::OnceCell;

static R2_CLIENT: OnceCell<Arc<R2Client>> = OnceCell::const_new();

pub struct R2Client {
    op: Operator,
}

impl R2Client {
    pub fn new() -> Result<Self, String> {
        let account_id =
            std::env::var("R2_ACCOUNT_ID").map_err(|_| "R2_ACCOUNT_ID not set".to_string())?;
        let access_key_id = std::env::var("R2_ACCESS_KEY_ID")
            .map_err(|_| "R2_ACCESS_KEY_ID not set".to_string())?;
        let secret_access_key = std::env::var("R2_SECRET_ACCESS_KEY")
            .map_err(|_| "R2_SECRET_ACCESS_KEY not set".to_string())?;
        let bucket = std::env::var("R2_BUCKET").map_err(|_| "R2_BUCKET not set".to_string())?;

        let endpoint = format!("https://{account_id}.r2.cloudflarestorage.com");

        let builder = S3::default()
            .bucket(&bucket)
            .region("auto")
            .endpoint(&endpoint)
            .access_key_id(&access_key_id)
            .secret_access_key(&secret_access_key);

        let op = Operator::new(builder)
            .map_err(|e| format!("Failed to build R2 operator: {e}"))?
            .finish();

        Ok(Self { op })
    }

    pub async fn get() -> Option<Arc<Self>> {
        R2_CLIENT
            .get_or_try_init(|| async {
                match Self::new() {
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

    pub async fn put_object(
        &self,
        key: &str,
        body: Vec<u8>,
        content_type: &str,
    ) -> Result<(), String> {
        self.op
            .write_with(key, body)
            .content_type(content_type)
            .await
            .map_err(|e| format!("Failed to put object to R2: {e}"))?;

        Ok(())
    }

    pub async fn delete_object(&self, key: &str) -> Result<(), String> {
        self.op
            .delete(key)
            .await
            .map_err(|e| format!("Failed to delete object from R2: {e}"))?;

        Ok(())
    }

    /// Delete all objects under a prefix (e.g., "projects/{id}/{ulid}/")
    pub async fn delete_prefix(&self, prefix: &str) -> Result<usize, String> {
        let entries = self
            .op
            .list(prefix)
            .await
            .map_err(|e| format!("Failed to list objects in R2: {e}"))?;

        let mut deleted = 0;
        for entry in entries {
            let path = entry.path();
            if !path.ends_with('/') {
                self.delete_object(path).await?;
                deleted += 1;
            }
        }

        Ok(deleted)
    }

    pub async fn object_exists(&self, key: &str) -> bool {
        self.op.stat(key).await.is_ok()
    }
}
