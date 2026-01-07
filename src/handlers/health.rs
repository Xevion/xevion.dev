use axum::{extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

use crate::state::AppState;

/// Health check endpoint - returns 200 if both DB and Bun are healthy
pub async fn health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let healthy = state.health_checker.check().await;

    if healthy {
        (StatusCode::OK, "OK")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "Unhealthy")
    }
}
