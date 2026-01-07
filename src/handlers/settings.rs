use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

use crate::{auth, db, state::AppState};

/// Get site settings (public endpoint)
pub async fn get_settings_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match db::get_site_settings(&state.pool).await {
        Ok(settings) => Json(settings).into_response(),
        Err(err) => {
            tracing::error!(error = %err, "Failed to fetch site settings");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to fetch settings"
                })),
            )
                .into_response()
        }
    }
}

/// Update site settings (requires authentication)
pub async fn update_settings_handler(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
    Json(payload): Json<db::UpdateSiteSettingsRequest>,
) -> impl IntoResponse {
    // Check authentication
    if auth::check_session(&state, &jar).is_none() {
        return auth::require_auth_response().into_response();
    }

    match db::update_site_settings(&state.pool, &payload).await {
        Ok(settings) => {
            // TODO: Invalidate ISR cache for homepage and affected routes when ISR is implemented
            // TODO: Add event log entry for settings update when events table is implemented
            tracing::info!("Site settings updated");
            Json(settings).into_response()
        }
        Err(err) => {
            tracing::error!(error = %err, "Failed to update site settings");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Internal server error",
                    "message": "Failed to update settings"
                })),
            )
                .into_response()
        }
    }
}
