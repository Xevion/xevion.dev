use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

use crate::{
    db,
    handlers::{CreateTagRequest, UpdateTagRequest},
    state::{AdminSession, AppError, AppResult, AppState, OptionNotFoundExt, SqlxResultExt},
    utils,
};

#[tracing::instrument(skip_all)]
pub async fn list_tags_handler(State(state): State<Arc<AppState>>) -> AppResult<impl IntoResponse> {
    let tags_with_counts = db::get_all_tags_with_counts(&state.pool).await?;
    let api_tags: Vec<db::ApiTagWithCount> = tags_with_counts
        .into_iter()
        .map(|(tag, count)| db::ApiTagWithCount {
            tag: tag.to_api_tag(),
            project_count: count,
        })
        .collect();
    Ok(Json(api_tags))
}

/// Create a new tag (requires authentication)
#[tracing::instrument(skip_all)]
pub async fn create_tag_handler(
    State(state): State<Arc<AppState>>,
    _session: AdminSession,
    Json(payload): Json<CreateTagRequest>,
) -> AppResult<impl IntoResponse> {
    if payload.name.trim().is_empty() {
        return Err(AppError::Validation("Tag name cannot be empty".into()));
    }
    if let Some(ref color) = payload.color
        && !utils::validate_hex_color(color)
    {
        return Err(AppError::Validation(
            "Invalid color format. Must be 6-character hex (e.g., '3b82f6')".into(),
        ));
    }

    let tag = db::create_tag(
        &state.pool,
        &payload.name,
        payload.slug.as_deref(),
        payload.icon.as_deref(),
        payload.color.as_deref(),
    )
    .await
    .conflict_on_unique("A tag with this name or slug already exists")?;

    state.isr_cache.invalidate("/").await;
    Ok((StatusCode::CREATED, Json(tag.to_api_tag())))
}

/// Get a tag by ref (UUID or slug) with associated projects
#[tracing::instrument(skip_all, fields(ref_str))]
pub async fn get_tag_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(ref_str): axum::extract::Path<String>,
) -> AppResult<impl IntoResponse> {
    let tag = db::get_tag_by_ref(&state.pool, &ref_str)
        .await?
        .or_not_found()?;
    let projects = db::get_projects_for_tag(&state.pool, tag.id).await?;
    let response = serde_json::json!({
        "tag": tag.to_api_tag(),
        "projects": projects.into_iter().map(|p| p.to_api_project()).collect::<Vec<_>>()
    });
    Ok(Json(response))
}

/// Update a tag (requires authentication)
#[tracing::instrument(skip_all, fields(ref_str))]
pub async fn update_tag_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(ref_str): axum::extract::Path<String>,
    _session: AdminSession,
    Json(payload): Json<UpdateTagRequest>,
) -> AppResult<impl IntoResponse> {
    if payload.name.trim().is_empty() {
        return Err(AppError::Validation("Tag name cannot be empty".into()));
    }
    if let Some(ref color) = payload.color
        && !utils::validate_hex_color(color)
    {
        return Err(AppError::Validation(
            "Invalid color format. Must be 6-character hex (e.g., '3b82f6')".into(),
        ));
    }

    let tag = db::get_tag_by_ref(&state.pool, &ref_str)
        .await?
        .or_not_found()?;

    let updated_tag = db::update_tag(
        &state.pool,
        tag.id,
        &payload.name,
        payload.slug.as_deref(),
        payload.icon.as_deref(),
        payload.color.as_deref(),
    )
    .await
    .conflict_on_unique("A tag with this name or slug already exists")?;

    state.isr_cache.invalidate("/").await;
    Ok(Json(updated_tag.to_api_tag()))
}

/// Delete a tag (requires authentication)
#[tracing::instrument(skip_all, fields(ref_str))]
pub async fn delete_tag_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(ref_str): axum::extract::Path<String>,
    _session: AdminSession,
) -> AppResult<impl IntoResponse> {
    let tag = db::get_tag_by_ref(&state.pool, &ref_str)
        .await?
        .or_not_found()?;

    db::delete_tag(&state.pool, tag.id).await?;
    tracing::info!(tag_id = %tag.id, tag_name = %tag.name, "Tag deleted");

    state.isr_cache.invalidate("/").await;
    Ok(Json(tag.to_api_tag()))
}

/// Get related tags by cooccurrence
#[tracing::instrument(skip_all, fields(ref_str))]
pub async fn get_related_tags_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(ref_str): axum::extract::Path<String>,
) -> AppResult<impl IntoResponse> {
    let tag = db::get_tag_by_ref(&state.pool, &ref_str)
        .await?
        .or_not_found()?;

    let related_tags = db::get_related_tags(&state.pool, tag.id, 10).await?;
    let api_related_tags: Vec<db::ApiRelatedTag> = related_tags
        .into_iter()
        .map(|(tag, count)| db::ApiRelatedTag {
            tag: tag.to_api_tag(),
            cooccurrence_count: count,
        })
        .collect();
    Ok(Json(api_related_tags))
}

/// Recalculate tag cooccurrence matrix (requires authentication)
#[tracing::instrument(skip_all)]
pub async fn recalculate_cooccurrence_handler(
    State(state): State<Arc<AppState>>,
    _session: AdminSession,
) -> AppResult<impl IntoResponse> {
    db::recalculate_tag_cooccurrence(&state.pool).await?;
    Ok(Json(serde_json::json!({
        "message": "Tag cooccurrence recalculated successfully"
    })))
}
