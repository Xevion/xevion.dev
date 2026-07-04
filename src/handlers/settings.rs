use axum::{
    Json,
    extract::{Multipart, State},
    response::IntoResponse,
};
use std::sync::Arc;
use ulid::Ulid;

use crate::{
    db,
    events::{self, EventLevel, EventType},
    handlers::media::extract_file,
    r2::R2Client,
    state::{AdminSession, AppError, AppResult, AppState},
};

#[tracing::instrument(skip_all)]
pub async fn get_settings_handler(
    State(state): State<Arc<AppState>>,
) -> AppResult<impl IntoResponse> {
    let settings = db::get_site_settings(&state.pool).await?;
    Ok(Json(settings))
}

#[tracing::instrument(skip_all)]
pub async fn update_settings_handler(
    State(state): State<Arc<AppState>>,
    session: AdminSession,
    Json(payload): Json<db::UpdateSiteSettingsRequest>,
) -> AppResult<impl IntoResponse> {
    let settings = db::update_site_settings(&state.pool, &payload).await?;
    tracing::info!("Site settings updated");
    events::log_event(
        &state.event_sender,
        EventType::SettingsUpdated,
        EventLevel::Info,
        Some("settings"),
        None,
        Some(&session.0.username),
        "Site settings updated".to_string(),
        None,
    );
    Ok(Json(settings))
}

/// Accepts multipart/form-data with a single PDF file; uploads it to R2 under
/// a fresh key (so old cached URLs keep working until the old object is
/// cleaned up) and records the new key on `site_identity`.
#[tracing::instrument(skip_all)]
pub async fn upload_resume_handler(
    State(state): State<Arc<AppState>>,
    session: AdminSession,
    mut multipart: Multipart,
) -> AppResult<impl IntoResponse> {
    let r2 = R2Client::get()
        .await
        .ok_or_else(|| AppError::ServiceUnavailable("Media storage is not configured".into()))?;

    let (_filename, content_type, data) = extract_file(&mut multipart)
        .await
        .map_err(AppError::validation)?
        .ok_or_else(|| AppError::validation("No file provided"))?;

    if content_type != "application/pdf" {
        return Err(AppError::validation(format!(
            "Content type '{content_type}' is not supported. Only application/pdf is accepted"
        )));
    }

    let key = format!("resume/{}.pdf", Ulid::new());
    r2.put_object(
        &key,
        data,
        "application/pdf",
        Some("public, max-age=31536000, immutable"),
    )
    .await
    .map_err(|e| AppError::Internal(format!("Failed to upload resume to storage: {e}")))?;

    let old_key = db::get_resume_key(&state.pool).await?;
    let identity = db::update_resume(&state.pool, &key).await?;

    if let Some(old_key) = old_key
        && let Err(e) = r2.delete_object(&old_key).await
    {
        tracing::warn!(error = %e, old_key = %old_key, "Failed to delete old resume (will be orphaned)");
    }

    tracing::info!(key = %key, "Resume uploaded successfully");
    events::log_event(
        &state.event_sender,
        EventType::ResumeUpdated,
        EventLevel::Info,
        Some("settings"),
        None,
        Some(&session.0.username),
        "Resume updated".to_string(),
        None,
    );
    Ok(Json(identity.to_api()))
}
