//! HTTP handlers for the CLI device-authorization flow.

use axum::{
    Json,
    extract::{Path, State},
    response::{
        IntoResponse, Sse,
        sse::{Event, KeepAlive},
    },
};
use futures::StreamExt;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::wrappers::WatchStream;
use ulid::Ulid;

use crate::{
    cli_auth::{
        ApproveRequest, CliAuthStatus, DenyRequest, StartRequest, StartResponse, VERIFICATION_PATH,
    },
    state::{AdminSession, AppError, AppResult, AppState},
};

/// Begin a CLI auth request. Unauthenticated: the browser approval step is what
/// authorizes it.
#[tracing::instrument(skip_all)]
pub async fn cli_auth_start_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<StartRequest>,
) -> AppResult<impl IntoResponse> {
    let started = state.cli_auth.start(payload.label);
    tracing::info!(request_id = %started.request_id, "CLI auth request started");

    Ok(Json(StartResponse {
        request_id: started.request_id.to_string(),
        user_code: started.user_code,
        verification_path: VERIFICATION_PATH.to_string(),
        expires_at: started.expires_at,
    }))
}

/// SSE stream the CLI waits on. Emits `pending` immediately, then the terminal
/// `approved`/`denied` event, after which the stream closes.
#[tracing::instrument(skip_all, fields(request_id = %request_id))]
pub async fn cli_auth_events_handler(
    State(state): State<Arc<AppState>>,
    Path(request_id): Path<String>,
) -> AppResult<Sse<impl futures::Stream<Item = Result<Event, Infallible>>>> {
    let request_id = Ulid::from_string(&request_id).map_err(|_| AppError::NotFound)?;
    let rx = state
        .cli_auth
        .subscribe(request_id)
        .ok_or(AppError::NotFound)?;

    let stream = WatchStream::new(rx)
        // Emit the terminal status, then end the stream.
        .scan(false, |stopped, status| {
            let result = if *stopped {
                None
            } else {
                if !matches!(status, CliAuthStatus::Pending) {
                    *stopped = true;
                }
                Some(status)
            };
            futures::future::ready(result)
        })
        .map(|status| Ok(Event::default().json_data(&status).unwrap_or_default()));

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// Approve a pending request. Requires an authenticated admin; mints a
/// long-lived CLI token bound to that admin and pushes it to the waiter.
#[tracing::instrument(skip_all)]
pub async fn cli_auth_approve_handler(
    State(state): State<Arc<AppState>>,
    session: AdminSession,
    Json(payload): Json<ApproveRequest>,
) -> AppResult<impl IntoResponse> {
    let request_id = Ulid::from_string(&payload.request_id).map_err(|_| AppError::NotFound)?;

    let info = state.cli_auth.info(request_id).ok_or(AppError::NotFound)?;
    if !info
        .user_code
        .eq_ignore_ascii_case(payload.user_code.trim())
    {
        return Err(AppError::validation("User code does not match"));
    }

    let (cli_session, token) = state
        .session_manager
        .create_cli_session(
            session.0.user_id,
            session.0.username.clone(),
            info.label.clone(),
        )
        .await?;

    if !state.cli_auth.approve(
        request_id,
        &payload.user_code,
        token,
        session.0.username.clone(),
    ) {
        // The request expired between the lookup and approval; roll back the token.
        let _ = state.session_manager.delete_session(cli_session.id).await;
        return Err(AppError::NotFound);
    }

    tracing::info!(
        request_id = %request_id,
        session_id = %cli_session.id,
        "CLI auth request approved"
    );

    Ok(Json(serde_json::json!({ "success": true })))
}

/// Deny a pending request. Requires an authenticated admin.
#[tracing::instrument(skip_all)]
pub async fn cli_auth_deny_handler(
    State(state): State<Arc<AppState>>,
    _session: AdminSession,
    Json(payload): Json<DenyRequest>,
) -> AppResult<impl IntoResponse> {
    let request_id = Ulid::from_string(&payload.request_id).map_err(|_| AppError::NotFound)?;
    state.cli_auth.deny(request_id);
    Ok(Json(serde_json::json!({ "success": true })))
}

/// Public view of a pending request, for the approval page to render before the
/// admin confirms. Unauthenticated so the page can show details immediately.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliAuthInfoResponse {
    pub user_code: String,
    pub label: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: time::OffsetDateTime,
}

#[tracing::instrument(skip_all)]
pub async fn cli_auth_info_handler(
    State(state): State<Arc<AppState>>,
    Path(request_id): Path<String>,
) -> AppResult<impl IntoResponse> {
    let request_id = Ulid::from_string(&request_id).map_err(|_| AppError::NotFound)?;
    let info = state.cli_auth.info(request_id).ok_or(AppError::NotFound)?;
    Ok(Json(CliAuthInfoResponse {
        user_code: info.user_code,
        label: info.label,
        expires_at: info.expires_at,
    }))
}
