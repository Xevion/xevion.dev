use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

use crate::{auth, db, github, handlers::AddProjectTagRequest, state::AppState};

/// List all projects - returns filtered data based on auth status
pub async fn projects_handler(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
) -> impl IntoResponse {
    let is_admin = auth::check_session(&state, &jar).is_some();

    if is_admin {
        // Admin view: return all projects with tags and media
        match db::get_all_projects_with_tags_admin(&state.pool).await {
            Ok(projects_with_tags) => {
                let response: Vec<db::ApiAdminProject> = projects_with_tags
                    .into_iter()
                    .map(|(project, tags, media)| project.to_api_admin_project(tags, media))
                    .collect();
                Json(response).into_response()
            }
            Err(err) => {
                tracing::error!(error = %err, "Failed to fetch admin projects");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "Internal server error",
                        "message": "Failed to fetch projects"
                    })),
                )
                    .into_response()
            }
        }
    } else {
        // Public view: return non-hidden projects with tags and media
        match db::get_public_projects_with_tags(&state.pool).await {
            Ok(projects_with_tags) => {
                let response: Vec<db::ApiAdminProject> = projects_with_tags
                    .into_iter()
                    .map(|(project, tags, media)| project.to_api_admin_project(tags, media))
                    .collect();
                Json(response).into_response()
            }
            Err(err) => {
                tracing::error!(error = %err, "Failed to fetch public projects");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "Internal server error",
                        "message": "Failed to fetch projects"
                    })),
                )
                    .into_response()
            }
        }
    }
}

/// Get a single project by ref (UUID or slug)
pub async fn get_project_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(ref_str): axum::extract::Path<String>,
    jar: axum_extra::extract::CookieJar,
) -> impl IntoResponse {
    let is_admin = auth::check_session(&state, &jar).is_some();

    match db::get_project_by_ref_with_tags(&state.pool, &ref_str).await {
        Ok(Some((project, tags, media))) => {
            // If project is hidden and user is not admin, return 404
            if project.status == db::ProjectStatus::Hidden && !is_admin {
                return (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "error": "Not found",
                        "message": "Project not found"
                    })),
                )
                    .into_response();
            }

            Json(project.to_api_admin_project(tags, media)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Not found",
                "message": "Project not found"
            })),
        )
            .into_response(),
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch project");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch project"
                })),
            )
                .into_response()
        }
    }
}

/// Create a new project (requires authentication)
pub async fn create_project_handler(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
    Json(payload): Json<db::CreateProjectRequest>,
) -> impl IntoResponse {
    // Check auth
    if auth::check_session(&state, &jar).is_none() {
        return auth::require_auth_response().into_response();
    }

    // Validate request
    if payload.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Validation error",
                "message": "Project name cannot be empty"
            })),
        )
            .into_response();
    }

    if payload.short_description.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Validation error",
                "message": "Project short description cannot be empty"
            })),
        )
            .into_response();
    }

    // Parse tag UUIDs
    let tag_ids: Result<Vec<uuid::Uuid>, _> = payload
        .tag_ids
        .iter()
        .map(|id| uuid::Uuid::parse_str(id))
        .collect();

    let tag_ids = match tag_ids {
        Ok(ids) => ids,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Validation error",
                    "message": "Invalid tag UUID format"
                })),
            )
                .into_response();
        }
    };

    // Create project
    let project = match db::create_project(
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
    {
        Ok(p) => p,
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "error": "Conflict",
                    "message": "A project with this slug already exists"
                })),
            )
                .into_response();
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to create project");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to create project"
                })),
            )
                .into_response();
        }
    };

    // Set tags
    if let Err(err) = db::set_project_tags(&state.pool, project.id, &tag_ids).await {
        tracing::error!(error = %err, project_id = %project.id, "Failed to set project tags");
    }

    // Fetch project with tags and media to return
    let (project, tags, media) = match db::get_project_by_id_with_tags(&state.pool, project.id)
        .await
    {
        Ok(Some(data)) => data,
        Ok(None) => {
            tracing::error!(project_id = %project.id, "Project not found after creation");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch created project"
                })),
            )
                .into_response();
        }
        Err(err) => {
            tracing::error!(error = %err, project_id = %project.id, "Failed to fetch created project");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch created project"
                })),
            )
                .into_response();
        }
    };

    tracing::info!(project_id = %project.id, project_name = %project.name, "Project created");

    // If project has a github_repo, add to scheduler for immediate checking
    if let Some(ref repo) = project.github_repo
        && let Some(scheduler) = github::get_scheduler()
    {
        scheduler.add_project(project.id, repo.clone());
        tracing::debug!(project_id = %project.id, repo = %repo, "Added project to GitHub scheduler");
    }

    // Invalidate cached pages that display projects
    state.isr_cache.invalidate("/").await;

    (
        StatusCode::CREATED,
        Json(project.to_api_admin_project(tags, media)),
    )
        .into_response()
}

/// Update an existing project (requires authentication)
pub async fn update_project_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(ref_str): axum::extract::Path<String>,
    jar: axum_extra::extract::CookieJar,
    Json(payload): Json<db::UpdateProjectRequest>,
) -> impl IntoResponse {
    // Check auth
    if auth::check_session(&state, &jar).is_none() {
        return auth::require_auth_response().into_response();
    }

    // Find project by ref (UUID or slug)
    let existing_project = match db::get_project_by_ref(&state.pool, &ref_str).await {
        Ok(Some(p)) => p,
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
            tracing::error!(error = %err, "Failed to fetch project");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch project"
                })),
            )
                .into_response();
        }
    };

    let project_id = existing_project.id;

    // Validate request
    if payload.name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Validation error",
                "message": "Project name cannot be empty"
            })),
        )
            .into_response();
    }

    if payload.short_description.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Validation error",
                "message": "Project short description cannot be empty"
            })),
        )
            .into_response();
    }

    // Parse tag UUIDs
    let tag_ids: Result<Vec<uuid::Uuid>, _> = payload
        .tag_ids
        .iter()
        .map(|id| uuid::Uuid::parse_str(id))
        .collect();

    let tag_ids = match tag_ids {
        Ok(ids) => ids,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Validation error",
                    "message": "Invalid tag UUID format"
                })),
            )
                .into_response();
        }
    };

    // Update project
    let project = match db::update_project(
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
    {
        Ok(p) => p,
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "error": "Conflict",
                    "message": "A project with this slug already exists"
                })),
            )
                .into_response();
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to update project");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to update project"
                })),
            )
                .into_response();
        }
    };

    // Update tags (smart diff)
    if let Err(err) = db::set_project_tags(&state.pool, project.id, &tag_ids).await {
        tracing::error!(error = %err, project_id = %project.id, "Failed to update project tags");
    }

    // Fetch updated project with tags and media
    let (project, tags, media) = match db::get_project_by_id_with_tags(&state.pool, project.id)
        .await
    {
        Ok(Some(data)) => data,
        Ok(None) => {
            tracing::error!(project_id = %project.id, "Project not found after update");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch updated project"
                })),
            )
                .into_response();
        }
        Err(err) => {
            tracing::error!(error = %err, project_id = %project.id, "Failed to fetch updated project");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch updated project"
                })),
            )
                .into_response();
        }
    };

    tracing::info!(project_id = %project.id, project_name = %project.name, "Project updated");

    // Update GitHub scheduler if github_repo changed
    if let Some(scheduler) = github::get_scheduler() {
        let old_repo = existing_project.github_repo.as_ref();
        let new_repo = project.github_repo.as_ref();

        match (old_repo, new_repo) {
            (None, Some(repo)) => {
                // Added github_repo - schedule for immediate check
                scheduler.add_project(project.id, repo.clone());
                tracing::debug!(project_id = %project.id, repo = %repo, "Added project to GitHub scheduler");
            }
            (Some(_), None) => {
                // Removed github_repo - remove from scheduler
                scheduler.remove_project(project.id);
                tracing::debug!(project_id = %project.id, "Removed project from GitHub scheduler");
            }
            (Some(old), Some(new)) if old != new => {
                // Changed github_repo - update scheduler
                scheduler.remove_project(project.id);
                scheduler.add_project(project.id, new.clone());
                tracing::debug!(project_id = %project.id, old_repo = %old, new_repo = %new, "Updated project in GitHub scheduler");
            }
            _ => {
                // No change to github_repo
            }
        }
    }

    // Invalidate cached pages that display projects
    state.isr_cache.invalidate("/").await;

    Json(project.to_api_admin_project(tags, media)).into_response()
}

/// Delete a project (requires authentication)
pub async fn delete_project_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(ref_str): axum::extract::Path<String>,
    jar: axum_extra::extract::CookieJar,
) -> impl IntoResponse {
    // Check auth
    if auth::check_session(&state, &jar).is_none() {
        return auth::require_auth_response().into_response();
    }

    // Fetch project before deletion to return it (lookup by UUID or slug)
    let (project, tags, media) = match db::get_project_by_ref_with_tags(&state.pool, &ref_str).await
    {
        Ok(Some(data)) => data,
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
            tracing::error!(error = %err, "Failed to fetch project before deletion");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to delete project"
                })),
            )
                .into_response();
        }
    };

    // Delete project (CASCADE handles tags and media)
    match db::delete_project(&state.pool, project.id).await {
        Ok(()) => {
            tracing::info!(project_id = %project.id, project_name = %project.name, "Project deleted");

            // Remove from GitHub scheduler if it had a github_repo
            if project.github_repo.is_some()
                && let Some(scheduler) = github::get_scheduler()
            {
                scheduler.remove_project(project.id);
            }

            // Invalidate cached pages that display projects
            state.isr_cache.invalidate("/").await;

            Json(project.to_api_admin_project(tags, media)).into_response()
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to delete project");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to delete project"
                })),
            )
                .into_response()
        }
    }
}

/// Get admin stats (requires authentication)
pub async fn get_admin_stats_handler(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
) -> impl IntoResponse {
    // Check auth
    if auth::check_session(&state, &jar).is_none() {
        return auth::require_auth_response().into_response();
    }

    match db::get_admin_stats(&state.pool).await {
        Ok(stats) => Json(stats).into_response(),
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch admin stats");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch statistics"
                })),
            )
                .into_response()
        }
    }
}

/// Get tags for a project
pub async fn get_project_tags_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(ref_str): axum::extract::Path<String>,
) -> impl IntoResponse {
    // Find project by ref (UUID or slug)
    let project = match db::get_project_by_ref(&state.pool, &ref_str).await {
        Ok(Some(p)) => p,
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
            tracing::error!(error = %err, "Failed to fetch project");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch project"
                })),
            )
                .into_response();
        }
    };

    match db::get_tags_for_project(&state.pool, project.id).await {
        Ok(tags) => {
            let api_tags: Vec<db::ApiTag> = tags.into_iter().map(|t| t.to_api_tag()).collect();
            Json(api_tags).into_response()
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch tags for project");
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

/// Add a tag to a project (requires authentication)
pub async fn add_project_tag_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(ref_str): axum::extract::Path<String>,
    jar: axum_extra::extract::CookieJar,
    Json(payload): Json<AddProjectTagRequest>,
) -> impl IntoResponse {
    if auth::check_session(&state, &jar).is_none() {
        return auth::require_auth_response().into_response();
    }

    // Find project by ref (UUID or slug)
    let project = match db::get_project_by_ref(&state.pool, &ref_str).await {
        Ok(Some(p)) => p,
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
            tracing::error!(error = %err, "Failed to fetch project");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch project"
                })),
            )
                .into_response();
        }
    };

    let tag_id = match uuid::Uuid::parse_str(&payload.tag_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid tag ID",
                    "message": "Tag ID must be a valid UUID"
                })),
            )
                .into_response();
        }
    };

    match db::add_tag_to_project(&state.pool, project.id, tag_id).await {
        Ok(()) => {
            // Invalidate cached pages - tags affect how projects are displayed
            state.isr_cache.invalidate("/").await;

            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "message": "Tag added to project"
                })),
            )
                .into_response()
        }
        Err(sqlx::Error::Database(db_err)) if db_err.is_foreign_key_violation() => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Not found",
                "message": "Project or tag not found"
            })),
        )
            .into_response(),
        Err(err) => {
            tracing::error!(error = %err, "Failed to add tag to project");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to add tag to project"
                })),
            )
                .into_response()
        }
    }
}

/// Remove a tag from a project (requires authentication)
pub async fn remove_project_tag_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path((ref_str, tag_ref)): axum::extract::Path<(String, String)>,
    jar: axum_extra::extract::CookieJar,
) -> impl IntoResponse {
    if auth::check_session(&state, &jar).is_none() {
        return auth::require_auth_response().into_response();
    }

    // Find project by ref (UUID or slug)
    let project = match db::get_project_by_ref(&state.pool, &ref_str).await {
        Ok(Some(p)) => p,
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
            tracing::error!(error = %err, "Failed to fetch project");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch project"
                })),
            )
                .into_response();
        }
    };

    // Find tag by ref (UUID or slug)
    let tag = match db::get_tag_by_ref(&state.pool, &tag_ref).await {
        Ok(Some(t)) => t,
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

    match db::remove_tag_from_project(&state.pool, project.id, tag.id).await {
        Ok(()) => {
            // Invalidate cached pages - tags affect how projects are displayed
            state.isr_cache.invalidate("/").await;

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "message": "Tag removed from project"
                })),
            )
                .into_response()
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to remove tag from project");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to remove tag from project"
                })),
            )
                .into_response()
        }
    }
}
