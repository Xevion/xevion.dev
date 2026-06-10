use axum::{
    Json,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use ulid::Ulid;
use uuid::Uuid;

use crate::{
    db, media_processing,
    r2::R2Client,
    state::{AdminSession, AppError, AppResult, AppState, OptionNotFoundExt},
};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderMediaRequest {
    pub media_ids: Vec<String>,
}

/// Accepts multipart/form-data; processes images into variants and uploads to R2.
#[tracing::instrument(skip_all, fields(ref_str))]
pub async fn upload_media_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(ref_str): axum::extract::Path<String>,
    _session: AdminSession,
    mut multipart: Multipart,
) -> AppResult<impl IntoResponse> {
    let project = db::get_project_by_ref(&state.pool, &ref_str)
        .await?
        .or_not_found()?;
    let project_id = project.id;

    let r2 = R2Client::get()
        .await
        .ok_or_else(|| AppError::ServiceUnavailable("Media storage is not configured".into()))?;

    let (filename, content_type, data) = extract_file(&mut multipart)
        .await
        .map_err(AppError::validation)?
        .ok_or_else(|| AppError::validation("No file provided"))?;

    let is_video = media_processing::is_supported_video(&content_type);
    let is_image = media_processing::is_supported_image(&content_type);

    if !is_video && !is_image {
        return Err(AppError::validation(format!(
            "Content type '{content_type}' is not supported. Supported: JPEG, PNG, GIF, WebP, AVIF, MP4, WebM"
        )));
    }

    let asset_id = Ulid::new();
    let r2_base_path = format!("projects/{project_id}/{asset_id}");

    if is_image {
        let processed = media_processing::process_image(&data, &filename)
            .map_err(|e| AppError::validation(format!("Failed to process image: {e}")))?;

        upload_image_variants(&r2, &r2_base_path, &processed)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to upload media to storage: {e}")))?;

        let original_ext = filename.rsplit('.').next().unwrap_or("jpg");
        let variants = serde_json::json!({
            "thumb": {
                "key": "thumb.webp",
                "width": processed.thumb.width,
                "height": processed.thumb.height
            },
            "medium": {
                "key": "medium.webp",
                "width": processed.medium.width,
                "height": processed.medium.height
            },
            "full": {
                "key": "full.webp",
                "width": processed.full.width,
                "height": processed.full.height
            },
            "original": {
                "key": format!("original.{original_ext}"),
                "width": processed.original.width,
                "height": processed.original.height,
                "mime": processed.original.mime
            }
        });

        let media = db::create_media(
            &state.pool,
            project_id,
            db::MediaType::Image,
            &filename,
            &r2_base_path,
            variants,
            Some(processed.original.width as i32),
            Some(processed.original.height as i32),
            data.len() as i64,
            Some(&processed.blurhash),
            None,
        )
        .await?;

        tracing::info!(
            media_id = %media.id,
            project_id = %project_id,
            filename = %filename,
            "Image uploaded successfully"
        );
        state.isr_cache.invalidate("/").await;

        Ok((StatusCode::CREATED, Json(media.to_api_media())))
    } else {
        let original_ext = match content_type.as_str() {
            "video/webm" => "webm",
            "video/quicktime" => "mov",
            _ => "mp4",
        };

        let video_key = format!("{r2_base_path}/original.{original_ext}");
        r2.put_object(&video_key, data.clone(), &content_type)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to upload video to storage: {e}")))?;

        let variants = serde_json::json!({
            "original": {
                "key": format!("original.{original_ext}"),
                "mime": content_type
            }
        });

        let media = db::create_media(
            &state.pool,
            project_id,
            db::MediaType::Video,
            &filename,
            &r2_base_path,
            variants,
            None,
            None,
            data.len() as i64,
            None,
            None,
        )
        .await?;

        tracing::info!(
            media_id = %media.id,
            project_id = %project_id,
            filename = %filename,
            "Video uploaded successfully"
        );
        state.isr_cache.invalidate("/").await;

        Ok((StatusCode::CREATED, Json(media.to_api_media())))
    }
}

/// Extract file from multipart form data
async fn extract_file(
    multipart: &mut Multipart,
) -> Result<Option<(String, String, Vec<u8>)>, String> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| format!("Failed to get field: {e}"))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" || name == "media" {
            let filename = field.file_name().unwrap_or("upload").to_string();
            let content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            let data = field
                .bytes()
                .await
                .map_err(|e| format!("Failed to read file data: {e}"))?
                .to_vec();
            return Ok(Some((filename, content_type, data)));
        }
    }
    Ok(None)
}

/// Upload all image variants to R2
async fn upload_image_variants(
    r2: &R2Client,
    base_path: &str,
    processed: &media_processing::ProcessedImage,
) -> Result<(), String> {
    r2.put_object(
        &format!("{base_path}/thumb.webp"),
        processed.thumb.data.clone(),
        "image/webp",
    )
    .await?;

    r2.put_object(
        &format!("{base_path}/medium.webp"),
        processed.medium.data.clone(),
        "image/webp",
    )
    .await?;

    r2.put_object(
        &format!("{base_path}/full.webp"),
        processed.full.data.clone(),
        "image/webp",
    )
    .await?;

    let original_ext = match processed.original.mime.as_str() {
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/avif" => "avif",
        _ => "jpg",
    };

    r2.put_object(
        &format!("{base_path}/original.{original_ext}"),
        processed.original.data.clone(),
        &processed.original.mime,
    )
    .await?;

    Ok(())
}

/// Get all media for a project
#[tracing::instrument(skip_all, fields(ref_str))]
pub async fn get_project_media_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(ref_str): axum::extract::Path<String>,
) -> AppResult<impl IntoResponse> {
    let project = db::get_project_by_ref(&state.pool, &ref_str)
        .await?
        .or_not_found()?;

    let media = db::get_media_for_project(&state.pool, project.id).await?;
    let response: Vec<db::ApiProjectMedia> = media.into_iter().map(|m| m.to_api_media()).collect();
    Ok(Json(response))
}

/// Delete a media item (requires authentication)
#[tracing::instrument(skip_all, fields(ref_str, media_id))]
pub async fn delete_media_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path((ref_str, media_id)): axum::extract::Path<(String, String)>,
    _session: AdminSession,
) -> AppResult<impl IntoResponse> {
    let project = db::get_project_by_ref(&state.pool, &ref_str)
        .await?
        .or_not_found()?;

    let media_id = Uuid::parse_str(&media_id)
        .map_err(|_| AppError::validation("Media ID must be a valid UUID"))?;

    let media = db::get_media_by_id(&state.pool, media_id)
        .await?
        .or_not_found()?;

    if media.project_id != project.id {
        return Err(AppError::NotFound);
    }

    // Delete files from R2 storage
    let r2_base_path = media.r2_base_path.clone();
    if let Some(r2) = R2Client::get().await {
        let prefix = format!("{}/", r2_base_path.trim_end_matches('/'));
        match r2.delete_prefix(&prefix).await {
            Ok(count) => {
                tracing::info!(
                    media_id = %media_id,
                    r2_prefix = %prefix,
                    deleted_count = count,
                    "Deleted R2 objects"
                );
            }
            Err(err) => {
                tracing::warn!(
                    error = %err,
                    media_id = %media_id,
                    r2_prefix = %prefix,
                    "Failed to delete R2 objects (will be orphaned)"
                );
            }
        }
    }

    let deleted = db::delete_media(&state.pool, media_id)
        .await?
        .or_not_found()?;

    tracing::info!(
        media_id = %media_id,
        project_id = %project.id,
        r2_base_path = %deleted.r2_base_path,
        "Media deleted from database"
    );
    state.isr_cache.invalidate("/").await;

    Ok(Json(deleted.to_api_media()))
}

/// Reorder media items for a project (requires authentication)
#[tracing::instrument(skip_all, fields(ref_str))]
pub async fn reorder_media_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(ref_str): axum::extract::Path<String>,
    _session: AdminSession,
    Json(payload): Json<ReorderMediaRequest>,
) -> AppResult<impl IntoResponse> {
    let project = db::get_project_by_ref(&state.pool, &ref_str)
        .await?
        .or_not_found()?;

    let media_ids: Vec<Uuid> = payload
        .media_ids
        .iter()
        .map(|id| Uuid::parse_str(id))
        .collect::<Result<_, _>>()
        .map_err(|_| AppError::field("mediaIds", "All media IDs must be valid UUIDs"))?;

    db::reorder_media(&state.pool, project.id, &media_ids).await?;

    let media = db::get_media_for_project(&state.pool, project.id).await?;
    state.isr_cache.invalidate("/").await;

    let response: Vec<db::ApiProjectMedia> = media.into_iter().map(|m| m.to_api_media()).collect();
    Ok(Json(response))
}
