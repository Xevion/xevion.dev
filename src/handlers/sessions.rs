//! Session management for the admin UI: list active sessions and revoke them.

use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use serde::Serialize;
use std::sync::Arc;
use ts_rs::TS;
use ulid::Ulid;

use crate::{
    auth::{Session, SessionType},
    state::{AdminSession, AppError, AppResult, AppState},
};

/// A session as shown in the admin sessions list.
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ApiSession {
    pub id: String,
    pub session_type: SessionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub label: Option<String>,
    pub created_at: String,
    pub last_active_at: String,
    pub expires_at: String,
    /// True for the session making this request (don't let the user lock themselves out blindly).
    pub current: bool,
}

impl ApiSession {
    fn from_session(session: &Session, current_id: Ulid) -> Self {
        let fmt = |dt: time::OffsetDateTime| {
            dt.format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default()
        };
        Self {
            id: session.id.to_string(),
            session_type: session.session_type,
            label: session.label.clone(),
            created_at: fmt(session.created_at),
            last_active_at: fmt(session.last_active_at),
            expires_at: fmt(session.expires_at),
            current: session.id == current_id,
        }
    }
}

/// List the authenticated admin's active sessions (browser + CLI), newest first.
#[tracing::instrument(skip_all)]
pub async fn list_sessions_handler(
    State(state): State<Arc<AppState>>,
    session: AdminSession,
) -> AppResult<impl IntoResponse> {
    let current_id = session.0.id;
    let sessions: Vec<ApiSession> = state
        .session_manager
        .list_user_sessions(session.0.user_id)
        .iter()
        .map(|s| ApiSession::from_session(s, current_id))
        .collect();
    Ok(Json(sessions))
}

/// Revoke a session by id. A session may only revoke sessions belonging to the
/// same admin user.
#[tracing::instrument(skip_all, fields(target = %target_id))]
pub async fn revoke_session_handler(
    State(state): State<Arc<AppState>>,
    session: AdminSession,
    Path(target_id): Path<String>,
) -> AppResult<impl IntoResponse> {
    let target = Ulid::from_string(&target_id).map_err(|_| AppError::NotFound)?;

    let owner = state
        .session_manager
        .get_session(target)
        .ok_or(AppError::NotFound)?;
    if owner.user_id != session.0.user_id {
        return Err(AppError::NotFound);
    }

    state.session_manager.delete_session(target).await?;
    tracing::info!(target = %target, "Session revoked");

    Ok(Json(serde_json::json!({ "success": true })))
}
