use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

use crate::{auth, state::AppState};

#[derive(serde::Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub username: String,
}

#[derive(serde::Serialize)]
pub struct SessionResponse {
    pub authenticated: bool,
    pub username: String,
}

/// Login handler - creates a new session
pub async fn api_login_handler(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(axum_extra::extract::CookieJar, Json<LoginResponse>), impl IntoResponse> {
    let user = match auth::get_admin_user(&state.pool, &payload.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Invalid credentials",
                    "message": "Username or password incorrect"
                })),
            ));
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch admin user");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to authenticate"
                })),
            ));
        }
    };

    let password_valid = match auth::verify_password(&payload.password, &user.password_hash) {
        Ok(valid) => valid,
        Err(err) => {
            tracing::error!(error = %err, "Failed to verify password");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to authenticate"
                })),
            ));
        }
    };

    if !password_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Invalid credentials",
                "message": "Username or password incorrect"
            })),
        ));
    }

    let session = match state
        .session_manager
        .create_session(user.id, user.username.clone())
        .await
    {
        Ok(session) => session,
        Err(err) => {
            tracing::error!(error = %err, "Failed to create session");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to create session"
                })),
            ));
        }
    };

    let cookie =
        axum_extra::extract::cookie::Cookie::build(("admin_session", session.id.to_string()))
            .path("/")
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .max_age(time::Duration::days(7))
            .build();

    let jar = jar.add(cookie);

    tracing::info!(username = %user.username, "User logged in");

    Ok((
        jar,
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
    if let Some(cookie) = jar.get("admin_session") {
        if let Ok(session_id) = ulid::Ulid::from_string(cookie.value()) {
            if let Err(e) = state.session_manager.delete_session(session_id).await {
                tracing::error!(error = %e, "Failed to delete session during logout");
            }
        }
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
) -> impl IntoResponse {
    let session_cookie = jar.get("admin_session");

    let session_id = session_cookie.and_then(|cookie| ulid::Ulid::from_string(cookie.value()).ok());

    let session = session_id.and_then(|id| state.session_manager.validate_session(id));

    match session {
        Some(session) => (
            StatusCode::OK,
            Json(SessionResponse {
                authenticated: true,
                username: session.username,
            }),
        )
            .into_response(),
        None => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Unauthorized",
                "message": "No valid session"
            })),
        )
            .into_response(),
    }
}
