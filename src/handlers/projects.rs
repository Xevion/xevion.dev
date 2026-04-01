use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

use crate::{
    auth, db,
    events::{self, EventLevel, EventType},
    github,
    handlers::{AddProjectTagRequest, CreateProjectRequest, UpdateProjectRequest},
    state::{AdminSession, AppError, AppResult, AppState, OptionNotFoundExt, SqlxResultExt},
};

#[tracing::instrument(skip_all)]
pub async fn projects_handler(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
) -> AppResult<impl IntoResponse> {
    let is_admin = auth::check_session(&state, &jar).is_some();

    let projects_with_tags = if is_admin {
        db::get_all_projects_with_tags_admin(&state.pool).await?
    } else {
        db::get_public_projects_with_tags(&state.pool).await?
    };

    let response: Vec<db::ApiAdminProject> = projects_with_tags
        .into_iter()
        .map(|(project, tags, media)| project.to_api_admin_project(tags, media))
        .collect::<AppResult<Vec<_>>>()?;
    Ok(Json(response))
}

/// Get a single project by ref (UUID or slug)
#[tracing::instrument(skip_all, fields(ref_str))]
pub async fn get_project_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(ref_str): axum::extract::Path<String>,
    jar: axum_extra::extract::CookieJar,
) -> AppResult<impl IntoResponse> {
    let is_admin = auth::check_session(&state, &jar).is_some();
    let (project, tags, media) = db::get_project_by_ref_with_tags(&state.pool, &ref_str)
        .await?
        .or_not_found()?;

    if project.status == db::ProjectStatus::Hidden && !is_admin {
        return Err(AppError::NotFound);
    }

    Ok(Json(project.to_api_admin_project(tags, media)?))
}

/// Create a new project (requires authentication)
#[tracing::instrument(skip_all)]
pub async fn create_project_handler(
    State(state): State<Arc<AppState>>,
    _session: AdminSession,
    Json(payload): Json<CreateProjectRequest>,
) -> AppResult<impl IntoResponse> {
    if payload.name.trim().is_empty() {
        return Err(AppError::Validation("Project name cannot be empty".into()));
    }
    if payload.short_description.trim().is_empty() {
        return Err(AppError::Validation(
            "Project short description cannot be empty".into(),
        ));
    }

    let tag_ids: Vec<uuid::Uuid> = payload
        .tag_ids
        .iter()
        .map(|id| uuid::Uuid::parse_str(id))
        .collect::<Result<_, _>>()
        .map_err(|_| AppError::Validation("Invalid tag UUID format".into()))?;

    let project = db::create_project(
        &state.pool,
        &payload.name,
        payload.slug.as_deref(),
        &payload.short_description,
        &payload.description,
        payload.status,
        payload.github_repo.as_deref(),
        payload.demo_url.as_deref(),
    )
    .await
    .conflict_on_unique("A project with this slug already exists")?;

    if let Err(err) = db::set_project_tags(&state.pool, project.id, &tag_ids).await {
        tracing::error!(error = %err, project_id = %project.id, "Failed to set project tags");
    }

    let (project, tags, media) = db::get_project_by_id_with_tags(&state.pool, project.id)
        .await?
        .or_not_found()?;

    tracing::info!(project_id = %project.id, project_name = %project.name, "Project created");
    events::log_event(
        &state.event_sender,
        EventType::ProjectCreated,
        EventLevel::Info,
        Some("project"),
        Some(project.id),
        Some(&_session.0.username),
        format!("Project created: {}", project.name),
        None,
    );

    if let Some(ref repo) = project.github_repo
        && let Some(scheduler) = github::get_scheduler()
    {
        scheduler.add_project(project.id, repo.clone());
        tracing::debug!(project_id = %project.id, repo = %repo, "Added project to GitHub scheduler");
    }

    state.isr_cache.invalidate("/").await;

    Ok((
        StatusCode::CREATED,
        Json(project.to_api_admin_project(tags, media)?),
    ))
}

/// Update an existing project (requires authentication)
#[tracing::instrument(skip_all, fields(ref_str))]
pub async fn update_project_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(ref_str): axum::extract::Path<String>,
    _session: AdminSession,
    Json(payload): Json<UpdateProjectRequest>,
) -> AppResult<impl IntoResponse> {
    let existing_project = db::get_project_by_ref(&state.pool, &ref_str)
        .await?
        .or_not_found()?;
    let project_id = existing_project.id;

    if payload.name.trim().is_empty() {
        return Err(AppError::Validation("Project name cannot be empty".into()));
    }
    if payload.short_description.trim().is_empty() {
        return Err(AppError::Validation(
            "Project short description cannot be empty".into(),
        ));
    }

    let tag_ids: Vec<uuid::Uuid> = payload
        .tag_ids
        .iter()
        .map(|id| uuid::Uuid::parse_str(id))
        .collect::<Result<_, _>>()
        .map_err(|_| AppError::Validation("Invalid tag UUID format".into()))?;

    let project = db::update_project(
        &state.pool,
        project_id,
        &payload.name,
        payload.slug.as_deref(),
        &payload.short_description,
        &payload.description,
        payload.status,
        payload.github_repo.as_deref(),
        payload.demo_url.as_deref(),
    )
    .await
    .conflict_on_unique("A project with this slug already exists")?;

    if let Err(err) = db::set_project_tags(&state.pool, project.id, &tag_ids).await {
        tracing::error!(error = %err, project_id = %project.id, "Failed to update project tags");
    }

    let (project, tags, media) = db::get_project_by_id_with_tags(&state.pool, project.id)
        .await?
        .or_not_found()?;

    tracing::info!(project_id = %project.id, project_name = %project.name, "Project updated");
    events::log_event(
        &state.event_sender,
        EventType::ProjectUpdated,
        EventLevel::Info,
        Some("project"),
        Some(project.id),
        Some(&_session.0.username),
        format!("Project updated: {}", project.name),
        None,
    );

    if let Some(scheduler) = github::get_scheduler() {
        let old_repo = existing_project.github_repo.as_ref();
        let new_repo = project.github_repo.as_ref();

        match (old_repo, new_repo) {
            (None, Some(repo)) => {
                scheduler.add_project(project.id, repo.clone());
                tracing::debug!(project_id = %project.id, repo = %repo, "Added project to GitHub scheduler");
            }
            (Some(_), None) => {
                scheduler.remove_project(project.id);
                tracing::debug!(project_id = %project.id, "Removed project from GitHub scheduler");
            }
            (Some(old), Some(new)) if old != new => {
                scheduler.remove_project(project.id);
                scheduler.add_project(project.id, new.clone());
                tracing::debug!(project_id = %project.id, old_repo = %old, new_repo = %new, "Updated project in GitHub scheduler");
            }
            _ => {}
        }
    }

    state.isr_cache.invalidate("/").await;

    Ok(Json(project.to_api_admin_project(tags, media)?))
}

/// Delete a project (requires authentication)
#[tracing::instrument(skip_all, fields(ref_str))]
pub async fn delete_project_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(ref_str): axum::extract::Path<String>,
    _session: AdminSession,
) -> AppResult<impl IntoResponse> {
    let (project, tags, media) = db::get_project_by_ref_with_tags(&state.pool, &ref_str)
        .await?
        .or_not_found()?;

    db::delete_project(&state.pool, project.id).await?;
    tracing::info!(project_id = %project.id, project_name = %project.name, "Project deleted");
    events::log_event(
        &state.event_sender,
        EventType::ProjectDeleted,
        EventLevel::Info,
        Some("project"),
        Some(project.id),
        Some(&_session.0.username),
        format!("Project deleted: {}", project.name),
        None,
    );

    if project.github_repo.is_some()
        && let Some(scheduler) = github::get_scheduler()
    {
        scheduler.remove_project(project.id);
    }

    state.isr_cache.invalidate("/").await;

    Ok(Json(project.to_api_admin_project(tags, media)?))
}

/// Get admin stats (requires authentication)
#[tracing::instrument(skip_all)]
pub async fn get_admin_stats_handler(
    State(state): State<Arc<AppState>>,
    _session: AdminSession,
) -> AppResult<impl IntoResponse> {
    let stats = db::get_admin_stats(&state.pool).await?;
    Ok(Json(stats))
}

/// Get tags for a project
#[tracing::instrument(skip_all, fields(ref_str))]
pub async fn get_project_tags_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(ref_str): axum::extract::Path<String>,
) -> AppResult<impl IntoResponse> {
    let project = db::get_project_by_ref(&state.pool, &ref_str)
        .await?
        .or_not_found()?;

    let tags = db::get_tags_for_project(&state.pool, project.id).await?;
    let api_tags: Vec<db::ApiTag> = tags.into_iter().map(|t| t.to_api_tag()).collect();
    Ok(Json(api_tags))
}

/// Add a tag to a project (requires authentication)
#[tracing::instrument(skip_all, fields(ref_str))]
pub async fn add_project_tag_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(ref_str): axum::extract::Path<String>,
    _session: AdminSession,
    Json(payload): Json<AddProjectTagRequest>,
) -> AppResult<impl IntoResponse> {
    let project = db::get_project_by_ref(&state.pool, &ref_str)
        .await?
        .or_not_found()?;

    let tag_id = uuid::Uuid::parse_str(&payload.tag_id)
        .map_err(|_| AppError::Validation("Tag ID must be a valid UUID".into()))?;

    db::add_tag_to_project(&state.pool, project.id, tag_id)
        .await
        .not_found_on_fk()?;

    events::log_event(
        &state.event_sender,
        EventType::ProjectTagAdded,
        EventLevel::Info,
        Some("project"),
        Some(project.id),
        Some(&_session.0.username),
        format!("Tag added to project: {}", project.name),
        None,
    );

    state.isr_cache.invalidate("/").await;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "message": "Tag added to project" })),
    ))
}

/// Remove a tag from a project (requires authentication)
#[tracing::instrument(skip_all, fields(ref_str, tag_ref))]
pub async fn remove_project_tag_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path((ref_str, tag_ref)): axum::extract::Path<(String, String)>,
    _session: AdminSession,
) -> AppResult<impl IntoResponse> {
    let project = db::get_project_by_ref(&state.pool, &ref_str)
        .await?
        .or_not_found()?;

    let tag = db::get_tag_by_ref(&state.pool, &tag_ref)
        .await?
        .or_not_found()?;

    db::remove_tag_from_project(&state.pool, project.id, tag.id).await?;

    events::log_event(
        &state.event_sender,
        EventType::ProjectTagRemoved,
        EventLevel::Info,
        Some("project"),
        Some(project.id),
        Some(&_session.0.username),
        format!("Tag removed from project: {}", project.name),
        None,
    );

    state.isr_cache.invalidate("/").await;

    Ok(Json(
        serde_json::json!({ "message": "Tag removed from project" }),
    ))
}
