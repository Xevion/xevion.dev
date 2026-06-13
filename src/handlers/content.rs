use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use std::sync::Arc;

use crate::{
    auth, db,
    events::{self, EventLevel, EventType},
    pm::{Doc, DocOp, OpError, PmError, generate_block_id},
    state::{AdminSession, AppError, AppResult, AppState, OptionNotFoundExt},
};

// A schema failure is about the submitted document, not the resolved project
// resource, so it's a 400 — shared by the ops path and the create/update path.
impl From<PmError> for AppError {
    fn from(err: PmError) -> Self {
        Self::validation(err.to_string())
    }
}

// Block-level failures are about the request payload, not the project resource
// (which we've already resolved), so they map to 4xx, never 404.
impl From<OpError> for AppError {
    fn from(err: OpError) -> Self {
        match err {
            OpError::NotFound(id) => Self::validation(format!("block \"{id}\" does not exist")),
            OpError::DuplicateId(id) => Self::Conflict(format!("block id \"{id}\" already exists")),
            OpError::SelfAnchor => Self::validation("a block cannot anchor to itself"),
            OpError::Invalid(schema_err) => schema_err.into(),
        }
    }
}

/// GET the block document. Hidden projects 404 for non-admins.
#[tracing::instrument(skip_all, fields(ref_str))]
pub async fn get_project_content_handler(
    State(state): State<Arc<AppState>>,
    Path(ref_str): Path<String>,
    jar: axum_extra::extract::CookieJar,
) -> AppResult<impl IntoResponse> {
    let is_admin = auth::check_session(&state, &jar).is_some();
    let project = db::get_project_by_ref(&state.pool, &ref_str)
        .await?
        .or_not_found()?;

    if project.status == db::ProjectStatus::Hidden && !is_admin {
        return Err(AppError::NotFound);
    }

    Ok(Json(
        Doc::from_stored(project.detail_content.as_ref()).into_inner(),
    ))
}

/// PATCH an atomic batch of block ops; returns the full updated document.
#[tracing::instrument(skip_all, fields(ref_str))]
pub async fn patch_project_content_handler(
    State(state): State<Arc<AppState>>,
    Path(ref_str): Path<String>,
    session: AdminSession,
    Json(ops): Json<Vec<DocOp>>,
) -> AppResult<impl IntoResponse> {
    let project = db::get_project_by_ref(&state.pool, &ref_str)
        .await?
        .or_not_found()?;

    let mut doc = Doc::from_stored(project.detail_content.as_ref());
    doc.apply_all(ops, generate_block_id)?;
    db::update_project_content(&state.pool, project.id, doc.to_stored().as_ref()).await?;

    tracing::info!(project_id = %project.id, "Project content updated");
    events::log_event(
        &state.event_sender,
        EventType::ProjectUpdated,
        EventLevel::Info,
        Some("project"),
        Some(project.id),
        Some(&session.0.username),
        format!("Project content updated: {}", project.name),
        None,
    );

    state.isr_cache.invalidate("/").await;
    state
        .isr_cache
        .invalidate(&format!("/projects/{}", project.slug))
        .await;

    Ok(Json(doc.into_inner()))
}
