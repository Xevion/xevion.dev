use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

/// Media type enum matching PostgreSQL enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "media_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Image,
    Video,
}

/// Database model for project media
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbProjectMedia {
    pub id: Uuid,
    pub project_id: Uuid,
    pub display_order: i32,
    pub media_type: MediaType,
    pub original_filename: String,
    pub r2_base_path: String,
    pub variants: serde_json::Value,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub size_bytes: i64,
    pub blurhash: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: OffsetDateTime,
}

/// Variant info for images
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageVariant {
    pub key: String,
    pub width: i32,
    pub height: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime: Option<String>,
}

/// Variant info for video poster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoOriginal {
    pub key: String,
    pub mime: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
}

/// API response for media variant with full URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMediaVariant {
    pub url: String,
    pub width: i32,
    pub height: i32,
}

/// API response for video original
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiVideoOriginal {
    pub url: String,
    pub mime: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
}

/// API response for media variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiMediaVariants {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb: Option<ApiMediaVariant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium: Option<ApiMediaVariant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full: Option<ApiMediaVariant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original: Option<ApiMediaVariant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poster: Option<ApiMediaVariant>,
    // For video original (different structure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<ApiVideoOriginal>,
}

/// Optional metadata stored with media
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focal_point: Option<FocalPoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocalPoint {
    pub x: f64,
    pub y: f64,
}

/// API response type for project media
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiProjectMedia {
    pub id: String,
    pub display_order: i32,
    pub media_type: MediaType,
    pub variants: ApiMediaVariants,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blurhash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<MediaMetadata>,
}

/// Base URL for R2 media storage
const R2_BASE_URL: &str = "https://media.xevion.dev";

impl DbProjectMedia {
    /// Convert database media to API response format
    pub fn to_api_media(&self) -> ApiProjectMedia {
        let variants = self.build_api_variants();
        let metadata = self
            .metadata
            .as_ref()
            .and_then(|m| serde_json::from_value(m.clone()).ok());

        ApiProjectMedia {
            id: self.id.to_string(),
            display_order: self.display_order,
            media_type: self.media_type,
            variants,
            blurhash: self.blurhash.clone(),
            metadata,
        }
    }

    fn build_api_variants(&self) -> ApiMediaVariants {
        let base_url = format!(
            "{}/{}",
            R2_BASE_URL,
            self.r2_base_path.trim_end_matches('/')
        );

        let mut variants = ApiMediaVariants {
            thumb: None,
            medium: None,
            full: None,
            original: None,
            poster: None,
            video: None,
        };

        // Parse the JSONB variants
        if let Some(obj) = self.variants.as_object() {
            // Handle image variants
            if let Some(thumb) = obj.get("thumb") {
                if let Ok(v) = serde_json::from_value::<ImageVariant>(thumb.clone()) {
                    variants.thumb = Some(ApiMediaVariant {
                        url: format!("{}/{}", base_url, v.key),
                        width: v.width,
                        height: v.height,
                    });
                }
            }

            if let Some(medium) = obj.get("medium") {
                if let Ok(v) = serde_json::from_value::<ImageVariant>(medium.clone()) {
                    variants.medium = Some(ApiMediaVariant {
                        url: format!("{}/{}", base_url, v.key),
                        width: v.width,
                        height: v.height,
                    });
                }
            }

            if let Some(full) = obj.get("full") {
                if let Ok(v) = serde_json::from_value::<ImageVariant>(full.clone()) {
                    variants.full = Some(ApiMediaVariant {
                        url: format!("{}/{}", base_url, v.key),
                        width: v.width,
                        height: v.height,
                    });
                }
            }

            // Handle original - could be image or video
            if let Some(original) = obj.get("original") {
                if self.media_type == MediaType::Video {
                    // Video original has different structure
                    if let Ok(v) = serde_json::from_value::<VideoOriginal>(original.clone()) {
                        variants.video = Some(ApiVideoOriginal {
                            url: format!("{}/{}", base_url, v.key),
                            mime: v.mime,
                            duration: v.duration,
                        });
                    }
                } else {
                    // Image original
                    if let Ok(v) = serde_json::from_value::<ImageVariant>(original.clone()) {
                        variants.original = Some(ApiMediaVariant {
                            url: format!("{}/{}", base_url, v.key),
                            width: v.width,
                            height: v.height,
                        });
                    }
                }
            }

            // Handle video poster
            if let Some(poster) = obj.get("poster") {
                if let Ok(v) = serde_json::from_value::<ImageVariant>(poster.clone()) {
                    variants.poster = Some(ApiMediaVariant {
                        url: format!("{}/{}", base_url, v.key),
                        width: v.width,
                        height: v.height,
                    });
                }
            }
        }

        variants
    }
}

// Database query functions

/// Get all media for a project, ordered by display_order
pub async fn get_media_for_project(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<DbProjectMedia>, sqlx::Error> {
    sqlx::query_as!(
        DbProjectMedia,
        r#"
        SELECT 
            id,
            project_id,
            display_order,
            media_type as "media_type: MediaType",
            original_filename,
            r2_base_path,
            variants,
            width,
            height,
            size_bytes,
            blurhash,
            metadata,
            created_at
        FROM project_media
        WHERE project_id = $1
        ORDER BY display_order ASC
        "#,
        project_id
    )
    .fetch_all(pool)
    .await
}

/// Get single media item by ID
pub async fn get_media_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<DbProjectMedia>, sqlx::Error> {
    sqlx::query_as!(
        DbProjectMedia,
        r#"
        SELECT 
            id,
            project_id,
            display_order,
            media_type as "media_type: MediaType",
            original_filename,
            r2_base_path,
            variants,
            width,
            height,
            size_bytes,
            blurhash,
            metadata,
            created_at
        FROM project_media
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
}

/// Get the next display order for a project's media
pub async fn get_next_display_order(pool: &PgPool, project_id: Uuid) -> Result<i32, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT COALESCE(MAX(display_order) + 1, 0) as "next_order!"
        FROM project_media
        WHERE project_id = $1
        "#,
        project_id
    )
    .fetch_one(pool)
    .await?;

    Ok(result.next_order)
}

/// Create a new media record
pub async fn create_media(
    pool: &PgPool,
    project_id: Uuid,
    media_type: MediaType,
    original_filename: &str,
    r2_base_path: &str,
    variants: serde_json::Value,
    width: Option<i32>,
    height: Option<i32>,
    size_bytes: i64,
    blurhash: Option<&str>,
    metadata: Option<serde_json::Value>,
) -> Result<DbProjectMedia, sqlx::Error> {
    let display_order = get_next_display_order(pool, project_id).await?;

    sqlx::query_as!(
        DbProjectMedia,
        r#"
        INSERT INTO project_media (
            project_id, display_order, media_type, original_filename,
            r2_base_path, variants, width, height, size_bytes, blurhash, metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING 
            id,
            project_id,
            display_order,
            media_type as "media_type: MediaType",
            original_filename,
            r2_base_path,
            variants,
            width,
            height,
            size_bytes,
            blurhash,
            metadata,
            created_at
        "#,
        project_id,
        display_order,
        media_type as MediaType,
        original_filename,
        r2_base_path,
        variants,
        width,
        height,
        size_bytes,
        blurhash,
        metadata
    )
    .fetch_one(pool)
    .await
}

/// Delete a media record
pub async fn delete_media(pool: &PgPool, id: Uuid) -> Result<Option<DbProjectMedia>, sqlx::Error> {
    // First get the media to return it
    let media = get_media_by_id(pool, id).await?;

    if media.is_some() {
        sqlx::query!("DELETE FROM project_media WHERE id = $1", id)
            .execute(pool)
            .await?;
    }

    Ok(media)
}

/// Reorder media for a project
/// Takes a list of media IDs in desired order and updates display_order accordingly
pub async fn reorder_media(
    pool: &PgPool,
    project_id: Uuid,
    media_ids: &[Uuid],
) -> Result<(), sqlx::Error> {
    // Use a transaction to ensure atomicity
    let mut tx = pool.begin().await?;

    // First, set all to negative values to avoid unique constraint conflicts
    for (i, id) in media_ids.iter().enumerate() {
        sqlx::query!(
            "UPDATE project_media SET display_order = $1 WHERE id = $2 AND project_id = $3",
            -(i as i32 + 1),
            id,
            project_id
        )
        .execute(&mut *tx)
        .await?;
    }

    // Then set to final positive values
    for (i, id) in media_ids.iter().enumerate() {
        sqlx::query!(
            "UPDATE project_media SET display_order = $1 WHERE id = $2 AND project_id = $3",
            i as i32,
            id,
            project_id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Update media metadata (focal point, alt text, etc.)
pub async fn update_media_metadata(
    pool: &PgPool,
    id: Uuid,
    metadata: serde_json::Value,
) -> Result<DbProjectMedia, sqlx::Error> {
    sqlx::query_as!(
        DbProjectMedia,
        r#"
        UPDATE project_media
        SET metadata = $2
        WHERE id = $1
        RETURNING 
            id,
            project_id,
            display_order,
            media_type as "media_type: MediaType",
            original_filename,
            r2_base_path,
            variants,
            width,
            height,
            size_bytes,
            blurhash,
            metadata,
            created_at
        "#,
        id,
        metadata
    )
    .fetch_one(pool)
    .await
}
