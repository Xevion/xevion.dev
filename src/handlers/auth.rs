use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

use crate::{
    auth,
    state::{AppError, AppResult, AppState},
};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub success: bool,
    pub username: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    pub authenticated: bool,
    pub username: String,
}

pub async fn api_login_handler(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
    Json(payload): Json<LoginRequest>,
) -> AppResult<impl IntoResponse> {
    let user = auth::get_admin_user(&state.pool, &payload.username)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    let password_valid = auth::verify_password(&payload.password, &user.password_hash)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if !password_valid {
        return Err(AppError::InvalidCredentials);
    }

    let session = state
        .session_manager
        .create_session(user.id, user.username.clone())
        .await?;

    let cookie =
        axum_extra::extract::cookie::Cookie::build(("admin_session", session.id.to_string()))
            .path("/")
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .max_age(time::Duration::days(7))
            .build();

    tracing::info!(username = %user.username, "User logged in");

    Ok((
        jar.add(cookie),
        Json(LoginResponse {
            success: true,
            username: user.username,
        }),
    ))
}

/// Logout handler - deletes the session
pub async fn api_logout_handler(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
) -> (axum_extra::extract::CookieJar, StatusCode) {
    if let Some(cookie) = jar.get("admin_session")
        && let Ok(session_id) = ulid::Ulid::from_string(cookie.value())
        && let Err(e) = state.session_manager.delete_session(session_id).await
    {
        tracing::error!(error = %e, "Failed to delete session during logout");
    }

    let cookie = axum_extra::extract::cookie::Cookie::build(("admin_session", ""))
        .path("/")
        .max_age(time::Duration::ZERO)
        .build();

    (jar.add(cookie), StatusCode::OK)
}

/// Session check handler - returns current session status
pub async fn api_session_handler(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
) -> AppResult<Json<SessionResponse>> {
    let session = auth::check_session(&state, &jar).ok_or(AppError::Unauthorized)?;

    Ok(Json(SessionResponse {
        authenticated: true,
        username: session.username,
    }))
}
