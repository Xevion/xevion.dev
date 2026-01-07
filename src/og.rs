use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};

use crate::{r2::R2Client, state::AppState};

/// Discriminated union matching TypeScript's `OGImageSpec` in web/src/lib/og-types.ts
///
/// IMPORTANT: Keep this in sync with the TypeScript definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum OGImageSpec {
    Index,
    Projects,
    Project { id: String },
}

impl OGImageSpec {
    /// Get the R2 storage key for this spec
    pub fn r2_key(&self) -> String {
        match self {
            OGImageSpec::Index => "og/index.png".to_string(),
            OGImageSpec::Projects => "og/projects.png".to_string(),
            OGImageSpec::Project { id } => format!("og/project/{id}.png"),
        }
    }
}

/// Generate an OG image by calling Bun's internal endpoint and upload to R2
#[tracing::instrument(skip(state), fields(r2_key))]
pub async fn generate_og_image(spec: &OGImageSpec, state: Arc<AppState>) -> Result<(), String> {
    let r2 = R2Client::get()
        .await
        .ok_or_else(|| "R2 client not available".to_string())?;

    let r2_key = spec.r2_key();
    tracing::Span::current().record("r2_key", &r2_key);

    // Call Bun's internal endpoint
    let bun_url = if state.downstream_url.starts_with('/') || state.downstream_url.starts_with("./")
    {
        "http://localhost/internal/ogp/generate".to_string()
    } else {
        format!("{}/internal/ogp/generate", state.downstream_url)
    };

    let client = state.unix_client.as_ref().unwrap_or(&state.http_client);

    let response = client
        .post(&bun_url)
        .json(spec)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("Failed to call Bun: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Bun returned status {status}: {error_text}"));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?
        .to_vec();

    r2.put_object(&r2_key, bytes)
        .await
        .map_err(|e| format!("Failed to upload to R2: {e}"))?;

    tracing::info!(r2_key, "OG image generated and uploaded");
    Ok(())
}

/// Check if an OG image exists in R2
pub async fn og_image_exists(spec: &OGImageSpec) -> bool {
    if let Some(r2) = R2Client::get().await {
        r2.object_exists(&spec.r2_key()).await
    } else {
        false
    }
}

/// Ensure an OG image exists, generating if necessary
pub async fn ensure_og_image(spec: &OGImageSpec, state: Arc<AppState>) -> Result<(), String> {
    if og_image_exists(spec).await {
        tracing::debug!(r2_key = spec.r2_key(), "OG image already exists");
        return Ok(());
    }
    generate_og_image(spec, state).await
}

/// Regenerate common OG images (index, projects) on server startup
/// Uses ensure_og_image to skip regeneration if images already exist
pub async fn regenerate_common_images(state: Arc<AppState>) {
    // Wait 2 seconds before starting
    tokio::time::sleep(Duration::from_secs(2)).await;

    tracing::info!("Ensuring common OG images exist");
    let specs = vec![OGImageSpec::Index, OGImageSpec::Projects];

    for spec in specs {
        match ensure_og_image(&spec, state.clone()).await {
            Ok(()) => {
                tracing::info!(r2_key = spec.r2_key(), "Common OG image ready");
            }
            Err(e) => {
                tracing::error!(r2_key = spec.r2_key(), error = %e, "Failed to ensure OG image");
            }
        }
    }

    tracing::info!("Finished ensuring common OG images");
}
