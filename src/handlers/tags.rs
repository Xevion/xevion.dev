use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

use crate::{
    auth, db,
    handlers::{CreateTagRequest, UpdateTagRequest},
    state::AppState,
    utils,
};

/// List all tags with project counts (public endpoint)
pub async fn list_tags_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match db::get_all_tags_with_counts(&state.pool).await {
        Ok(tags_with_counts) => {
            let api_tags: Vec<db::ApiTagWithCount> = tags_with_counts
                .into_iter()
                .map(|(tag, count)| db::ApiTagWithCount {
                    tag: tag.to_api_tag(),
                    project_count: count,
                })
                .collect();
            Json(api_tags).into_response()
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch tags");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch tags"
                })),
            )
                .into_response()
        }
    }
}

/// Create a new tag (requires authentication)
pub async fn create_tag_handler(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
    Json(payload): Json<CreateTagRequest>,
) -> impl IntoResponse {
    if auth::check_session(&state, &jar).is_none() {
        return auth::require_auth_response().into_response();
    }
    if payload.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Validation error",
                "message": "Tag name cannot be empty"
            })),
        )
            .into_response();
    }

    // Validate color if provided
    if let Some(ref color) = payload.color {
        if !utils::validate_hex_color(color) {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Validation error",
                    "message": "Invalid color format. Must be 6-character hex (e.g., '3b82f6')"
                })),
            )
                .into_response();
        }
    }

    match db::create_tag(
        &state.pool,
        &payload.name,
        payload.slug.as_deref(),
        payload.icon.as_deref(),
        payload.color.as_deref(),
    )
    .await
    {
        Ok(tag) => (StatusCode::CREATED, Json(tag.to_api_tag())).into_response(),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "Conflict",
                "message": "A tag with this name or slug already exists"
            })),
        )
            .into_response(),
        Err(err) => {
            tracing::error!(error = %err, "Failed to create tag");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to create tag"
                })),
            )
                .into_response()
        }
    }
}

/// Get a tag by slug with associated projects
pub async fn get_tag_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(slug): axum::extract::Path<String>,
) -> impl IntoResponse {
    match db::get_tag_by_slug(&state.pool, &slug).await {
        Ok(Some(tag)) => match db::get_projects_for_tag(&state.pool, tag.id).await {
            Ok(projects) => {
                let response = serde_json::json!({
                    "tag": tag.to_api_tag(),
                    "projects": projects.into_iter().map(|p| p.to_api_project()).collect::<Vec<_>>()
                });
                Json(response).into_response()
            }
            Err(err) => {
                tracing::error!(error = %err, "Failed to fetch projects for tag");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "Internal server error",
                        "message": "Failed to fetch projects"
                    })),
                )
                    .into_response()
            }
        },
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Not found",
                "message": "Tag not found"
            })),
        )
            .into_response(),
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch tag");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch tag"
                })),
            )
                .into_response()
        }
    }
}

/// Update a tag (requires authentication)
pub async fn update_tag_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(slug): axum::extract::Path<String>,
    jar: axum_extra::extract::CookieJar,
    Json(payload): Json<UpdateTagRequest>,
) -> impl IntoResponse {
    if auth::check_session(&state, &jar).is_none() {
        return auth::require_auth_response().into_response();
    }
    if payload.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Validation error",
                "message": "Tag name cannot be empty"
            })),
        )
            .into_response();
    }

    // Validate color if provided
    if let Some(ref color) = payload.color {
        if !utils::validate_hex_color(color) {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Validation error",
                    "message": "Invalid color format. Must be 6-character hex (e.g., '3b82f6')"
                })),
            )
                .into_response();
        }
    }

    let tag = match db::get_tag_by_slug(&state.pool, &slug).await {
        Ok(Some(tag)) => tag,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Not found",
                    "message": "Tag not found"
                })),
            )
                .into_response();
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch tag");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch tag"
                })),
            )
                .into_response();
        }
    };

    match db::update_tag(
        &state.pool,
        tag.id,
        &payload.name,
        payload.slug.as_deref(),
        payload.icon.as_deref(),
        payload.color.as_deref(),
    )
    .await
    {
        Ok(updated_tag) => Json(updated_tag.to_api_tag()).into_response(),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "Conflict",
                "message": "A tag with this name or slug already exists"
            })),
        )
            .into_response(),
        Err(err) => {
            tracing::error!(error = %err, "Failed to update tag");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to update tag"
                })),
            )
                .into_response()
        }
    }
}

/// Get related tags by cooccurrence
pub async fn get_related_tags_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(slug): axum::extract::Path<String>,
) -> impl IntoResponse {
    let tag = match db::get_tag_by_slug(&state.pool, &slug).await {
        Ok(Some(tag)) => tag,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Not found",
                    "message": "Tag not found"
                })),
            )
                .into_response();
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch tag");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch tag"
                })),
            )
                .into_response();
        }
    };

    match db::get_related_tags(&state.pool, tag.id, 10).await {
        Ok(related_tags) => {
            let api_related_tags: Vec<db::ApiRelatedTag> = related_tags
                .into_iter()
                .map(|(tag, count)| db::ApiRelatedTag {
                    tag: tag.to_api_tag(),
                    cooccurrence_count: count,
                })
                .collect();
            Json(api_related_tags).into_response()
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch related tags");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch related tags"
                })),
            )
                .into_response()
        }
    }
}

/// Recalculate tag cooccurrence matrix (requires authentication)
pub async fn recalculate_cooccurrence_handler(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
) -> impl IntoResponse {
    if auth::check_session(&state, &jar).is_none() {
        return auth::require_auth_response().into_response();
    }
    match db::recalculate_tag_cooccurrence(&state.pool).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "Tag cooccurrence recalculated successfully"
            })),
        )
            .into_response(),
        Err(err) => {
            tracing::error!(error = %err, "Failed to recalculate cooccurrence");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to recalculate cooccurrence"
                })),
            )
                .into_response()
        }
    }
}
