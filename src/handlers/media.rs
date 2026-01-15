use axum::{
    Json,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;
use ulid::Ulid;
use uuid::Uuid;

use crate::{auth, db, media_processing, r2::R2Client, state::AppState};

/// Request type for reordering media
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReorderMediaRequest {
    /// Media IDs in desired order
    pub media_ids: Vec<String>,
}

/// Upload media for a project (requires authentication)
///
/// Accepts multipart/form-data with a single file field.
/// Images are processed into variants (thumb, medium, full) and uploaded to R2.
pub async fn upload_media_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(project_id): axum::extract::Path<String>,
    jar: axum_extra::extract::CookieJar,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // Check auth
    if auth::check_session(&state, &jar).is_none() {
        return auth::require_auth_response().into_response();
    }

    let project_id = match Uuid::parse_str(&project_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid project ID",
                    "message": "Project ID must be a valid UUID"
                })),
            )
                .into_response();
        }
    };

    // Verify project exists
    match db::get_project_by_id(&state.pool, project_id).await {
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Not found",
                    "message": "Project not found"
                })),
            )
                .into_response();
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to check project existence");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to verify project"
                })),
            )
                .into_response();
        }
        Ok(Some(_)) => {}
    }

    // Get R2 client
    let r2 = match R2Client::get().await {
        Some(r2) => r2,
        None => {
            tracing::error!("R2 client not available");
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "Service unavailable",
                    "message": "Media storage is not configured"
                })),
            )
                .into_response();
        }
    };

    // Extract file from multipart
    let (filename, content_type, data) = match extract_file(&mut multipart).await {
        Ok(Some(file)) => file,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Bad request",
                    "message": "No file provided"
                })),
            )
                .into_response();
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to extract file from multipart");
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Bad request",
                    "message": format!("Failed to read upload: {err}")
                })),
            )
                .into_response();
        }
    };

    // Determine media type and process
    let is_video = media_processing::is_supported_video(&content_type);
    let is_image = media_processing::is_supported_image(&content_type);

    if !is_video && !is_image {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Unsupported format",
                "message": format!("Content type '{}' is not supported. Supported: JPEG, PNG, GIF, WebP, AVIF, MP4, WebM", content_type)
            })),
        )
            .into_response();
    }

    // Generate unique asset ID
    let asset_id = Ulid::new();
    let r2_base_path = format!("projects/{project_id}/{asset_id}");

    if is_image {
        // Process image
        let processed = match media_processing::process_image(&data, &filename) {
            Ok(p) => p,
            Err(err) => {
                tracing::error!(error = %err, filename = %filename, "Failed to process image");
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "Processing failed",
                        "message": format!("Failed to process image: {err}")
                    })),
                )
                    .into_response();
            }
        };

        // Upload all variants to R2
        if let Err(err) = upload_image_variants(&r2, &r2_base_path, &processed).await {
            tracing::error!(error = %err, "Failed to upload image variants to R2");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Upload failed",
                    "message": "Failed to upload media to storage"
                })),
            )
                .into_response();
        }

        // Build variants JSON
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

        // Create database record
        match db::create_media(
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
        .await
        {
            Ok(media) => {
                tracing::info!(
                    media_id = %media.id,
                    project_id = %project_id,
                    filename = %filename,
                    "Image uploaded successfully"
                );

                // Invalidate cache
                state.isr_cache.invalidate("/").await;

                (StatusCode::CREATED, Json(media.to_api_media())).into_response()
            }
            Err(err) => {
                tracing::error!(error = %err, "Failed to create media record");
                // TODO: Clean up R2 files on DB failure
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "Database error",
                        "message": "Failed to save media record"
                    })),
                )
                    .into_response()
            }
        }
    } else {
        // Video upload - basic support (no transcoding, ffmpeg poster extraction optional)
        let original_ext = match content_type.as_str() {
            "video/mp4" => "mp4",
            "video/webm" => "webm",
            "video/quicktime" => "mov",
            _ => "mp4",
        };

        // Upload original video
        let video_key = format!("{r2_base_path}/original.{original_ext}");
        if let Err(err) = r2.put_object(&video_key, data.clone(), &content_type).await {
            tracing::error!(error = %err, "Failed to upload video to R2");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Upload failed",
                    "message": "Failed to upload video to storage"
                })),
            )
                .into_response();
        }

        // Build variants JSON (video only has original for now)
        let variants = serde_json::json!({
            "original": {
                "key": format!("original.{original_ext}"),
                "mime": content_type
            }
        });

        // Create database record
        match db::create_media(
            &state.pool,
            project_id,
            db::MediaType::Video,
            &filename,
            &r2_base_path,
            variants,
            None, // Video dimensions would require ffprobe
            None,
            data.len() as i64,
            None, // No blurhash without poster
            None,
        )
        .await
        {
            Ok(media) => {
                tracing::info!(
                    media_id = %media.id,
                    project_id = %project_id,
                    filename = %filename,
                    "Video uploaded successfully"
                );

                // Invalidate cache
                state.isr_cache.invalidate("/").await;

                (StatusCode::CREATED, Json(media.to_api_media())).into_response()
            }
            Err(err) => {
                tracing::error!(error = %err, "Failed to create media record");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "Database error",
                        "message": "Failed to save media record"
                    })),
                )
                    .into_response()
            }
        }
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

        // Accept 'file' or 'media' field names
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
    // Upload thumb
    r2.put_object(
        &format!("{base_path}/thumb.webp"),
        processed.thumb.data.clone(),
        "image/webp",
    )
    .await?;

    // Upload medium
    r2.put_object(
        &format!("{base_path}/medium.webp"),
        processed.medium.data.clone(),
        "image/webp",
    )
    .await?;

    // Upload full
    r2.put_object(
        &format!("{base_path}/full.webp"),
        processed.full.data.clone(),
        "image/webp",
    )
    .await?;

    // Upload original (preserve format)
    let original_ext = match processed.original.mime.as_str() {
        "image/jpeg" => "jpg",
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
pub async fn get_project_media_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(project_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let project_id = match Uuid::parse_str(&project_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid project ID",
                    "message": "Project ID must be a valid UUID"
                })),
            )
                .into_response();
        }
    };

    // Verify project exists
    match db::get_project_by_id(&state.pool, project_id).await {
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Not found",
                    "message": "Project not found"
                })),
            )
                .into_response();
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to check project existence");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch project"
                })),
            )
                .into_response();
        }
        Ok(Some(_)) => {}
    }

    match db::get_media_for_project(&state.pool, project_id).await {
        Ok(media) => {
            let response: Vec<db::ApiProjectMedia> =
                media.into_iter().map(|m| m.to_api_media()).collect();
            Json(response).into_response()
        }
        Err(err) => {
            tracing::error!(error = %err, project_id = %project_id, "Failed to fetch project media");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch media"
                })),
            )
                .into_response()
        }
    }
}

/// Delete a media item (requires authentication)
pub async fn delete_media_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path((project_id, media_id)): axum::extract::Path<(String, String)>,
    jar: axum_extra::extract::CookieJar,
) -> impl IntoResponse {
    // Check auth
    if auth::check_session(&state, &jar).is_none() {
        return auth::require_auth_response().into_response();
    }

    let project_id = match Uuid::parse_str(&project_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid project ID",
                    "message": "Project ID must be a valid UUID"
                })),
            )
                .into_response();
        }
    };

    let media_id = match Uuid::parse_str(&media_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid media ID",
                    "message": "Media ID must be a valid UUID"
                })),
            )
                .into_response();
        }
    };

    // Get media first to verify it belongs to the project
    match db::get_media_by_id(&state.pool, media_id).await {
        Ok(Some(media)) => {
            if media.project_id != project_id {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "error": "Not found",
                        "message": "Media not found for this project"
                    })),
                )
                    .into_response();
            }

            // Delete files from R2 storage
            let r2_base_path = media.r2_base_path.clone();
            if let Some(r2) = R2Client::get().await {
                // Delete all files under the media's R2 prefix
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
                        // Log but don't fail - DB record deletion is more important
                        tracing::warn!(
                            error = %err,
                            media_id = %media_id,
                            r2_prefix = %prefix,
                            "Failed to delete R2 objects (will be orphaned)"
                        );
                    }
                }
            }

            match db::delete_media(&state.pool, media_id).await {
                Ok(Some(deleted)) => {
                    tracing::info!(
                        media_id = %media_id,
                        project_id = %project_id,
                        r2_base_path = %deleted.r2_base_path,
                        "Media deleted from database"
                    );

                    // Invalidate cache since project data changed
                    state.isr_cache.invalidate("/").await;

                    Json(deleted.to_api_media()).into_response()
                }
                Ok(None) => (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "error": "Not found",
                        "message": "Media not found"
                    })),
                )
                    .into_response(),
                Err(err) => {
                    tracing::error!(error = %err, "Failed to delete media");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": "Internal server error",
                            "message": "Failed to delete media"
                        })),
                    )
                        .into_response()
                }
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Not found",
                "message": "Media not found"
            })),
        )
            .into_response(),
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch media");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch media"
                })),
            )
                .into_response()
        }
    }
}

/// Reorder media items for a project (requires authentication)
pub async fn reorder_media_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(project_id): axum::extract::Path<String>,
    jar: axum_extra::extract::CookieJar,
    Json(payload): Json<ReorderMediaRequest>,
) -> impl IntoResponse {
    // Check auth
    if auth::check_session(&state, &jar).is_none() {
        return auth::require_auth_response().into_response();
    }

    let project_id = match Uuid::parse_str(&project_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid project ID",
                    "message": "Project ID must be a valid UUID"
                })),
            )
                .into_response();
        }
    };

    // Parse media IDs
    let media_ids: Result<Vec<Uuid>, _> = payload
        .media_ids
        .iter()
        .map(|id| Uuid::parse_str(id))
        .collect();

    let media_ids = match media_ids {
        Ok(ids) => ids,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid media ID",
                    "message": "All media IDs must be valid UUIDs"
                })),
            )
                .into_response();
        }
    };

    // Verify project exists
    match db::get_project_by_id(&state.pool, project_id).await {
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Not found",
                    "message": "Project not found"
                })),
            )
                .into_response();
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to check project existence");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to verify project"
                })),
            )
                .into_response();
        }
        Ok(Some(_)) => {}
    }

    // Reorder media
    match db::reorder_media(&state.pool, project_id, &media_ids).await {
        Ok(()) => {
            // Fetch updated media list
            match db::get_media_for_project(&state.pool, project_id).await {
                Ok(media) => {
                    // Invalidate cache since project data changed
                    state.isr_cache.invalidate("/").await;

                    let response: Vec<db::ApiProjectMedia> =
                        media.into_iter().map(|m| m.to_api_media()).collect();
                    Json(response).into_response()
                }
                Err(err) => {
                    tracing::error!(error = %err, "Failed to fetch updated media");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": "Internal server error",
                            "message": "Reorder succeeded but failed to fetch updated list"
                        })),
                    )
                        .into_response()
                }
            }
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to reorder media");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to reorder media"
                })),
            )
                .into_response()
        }
    }
}

// TODO: Upload handler requires multipart form handling and image processing
// This will be implemented when we add the upload functionality
// For now, media records can only be created programmatically or via seeding
