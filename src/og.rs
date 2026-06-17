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
            Self::Index => "og/index.png".to_string(),
            Self::Projects => "og/projects.png".to_string(),
            Self::Project { id } => format!("og/project/{id}.png"),
        }
    }

    /// `Cache-Control` to store on the R2 object.
    ///
    /// Project cards are served with a `?v=<updated_at>` cache-bust in the URL
    /// (see `getOGImageUrl` in `web/src/lib/og-types.ts`), so a fresh edit yields
    /// a fresh URL — the object itself can be pinned `immutable` for a year.
    /// Index/projects cards share a fixed, un-busted URL, so they only cache for a
    /// day to avoid serving a stale card long after the underlying content changes.
    pub const fn cache_control(&self) -> &'static str {
        match self {
            Self::Project { .. } => "public, max-age=31536000, immutable",
            Self::Index | Self::Projects => "public, max-age=86400",
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
    let response = state
        .client
        .post("/internal/ogp/generate")
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

    r2.put_object(&r2_key, bytes, "image/png", Some(spec.cache_control()))
        .await
        .map_err(|e| format!("Failed to upload to R2: {e}"))?;

    tracing::info!(r2_key, "OG image generated and uploaded");
    crate::events::log_event(
        &state.event_sender,
        crate::events::EventType::OgImageGenerated,
        crate::events::EventLevel::Info,
        Some("system"),
        None,
        None,
        format!("OG image generated: {r2_key}"),
        None,
    );
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
/// Uses `ensure_og_image` to skip regeneration if images already exist
pub async fn regenerate_common_images(state: Arc<AppState>) {
    // Wait 2 seconds before starting
    tokio::time::sleep(Duration::from_secs(2)).await;

    tracing::debug!("Checking common OG images");
    let specs = vec![OGImageSpec::Index, OGImageSpec::Projects];

    let mut ready = Vec::new();

    for spec in &specs {
        match ensure_og_image(spec, state.clone()).await {
            Ok(()) => ready.push(spec.r2_key()),
            Err(e) => {
                tracing::error!(r2_key = spec.r2_key(), error = %e, "OG image failed");
                crate::events::log_event(
                    &state.event_sender,
                    crate::events::EventType::OgImageFailed,
                    crate::events::EventLevel::Error,
                    Some("system"),
                    None,
                    None,
                    format!("OG image failed: {}: {e}", spec.r2_key()),
                    None,
                );
            }
        }
    }

    tracing::info!(images = ?ready, "Common OG images ready");

    ensure_project_images(state).await;
}

/// Backfill any missing per-project OG cards on startup.
///
/// New/edited projects get their card generated inline by the mutation handlers
/// (see [`spawn_project_og`]); this covers projects that predate that wiring or
/// whose upload was lost, so existing cards aren't stuck 404ing until the next
/// edit. [`ensure_og_image`] skips projects whose card already exists, so repeat
/// startups only pay for existence checks.
async fn ensure_project_images(state: Arc<AppState>) {
    let projects = match crate::db::get_all_projects_admin(&state.pool).await {
        Ok(projects) => projects,
        Err(e) => {
            tracing::error!(error = %e, "Failed to list projects for OG backfill");
            return;
        }
    };

    let mut generated = 0u32;
    for project in &projects {
        let spec = OGImageSpec::Project {
            id: project.id.to_string(),
        };
        match ensure_og_image(&spec, state.clone()).await {
            Ok(()) => generated += 1,
            Err(e) => {
                tracing::error!(project_id = %project.id, error = %e, "Project OG backfill failed");
            }
        }
    }

    tracing::info!(
        total = projects.len(),
        ensured = generated,
        "Project OG images backfilled"
    );
}

/// Fire-and-forget (re)generation of a project's OG card after a create/update.
///
/// Spawned so the mutating request isn't blocked on Satori/R2; failures are
/// logged, never surfaced. The card is overwritten in place at `og/project/{id}.png`,
/// and the page's `?v=<updated_at>` cache-bust makes the refreshed image visible.
pub fn spawn_project_og(state: Arc<AppState>, project_id: uuid::Uuid) {
    tokio::spawn(async move {
        let spec = OGImageSpec::Project {
            id: project_id.to_string(),
        };
        if let Err(e) = generate_og_image(&spec, state).await {
            tracing::warn!(project_id = %project_id, error = %e, "Failed to generate project OG image");
        }
    });
}

/// Fire-and-forget removal of a deleted project's OG card from R2 (best-effort).
pub fn spawn_delete_project_og(project_id: uuid::Uuid) {
    tokio::spawn(async move {
        let Some(r2) = R2Client::get().await else {
            return;
        };
        let key = OGImageSpec::Project {
            id: project_id.to_string(),
        }
        .r2_key();
        if let Err(e) = r2.delete_object(&key).await {
            tracing::warn!(project_id = %project_id, error = %e, "Failed to delete project OG image");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_images_are_immutable_versioned_cards_cache_daily() {
        let project = OGImageSpec::Project {
            id: "abc".to_string(),
        };
        assert_eq!(
            project.cache_control(),
            "public, max-age=31536000, immutable"
        );
        // Un-busted shared cards revalidate daily rather than pin for a year.
        assert_eq!(OGImageSpec::Index.cache_control(), "public, max-age=86400");
        assert_eq!(
            OGImageSpec::Projects.cache_control(),
            "public, max-age=86400"
        );
    }
}
