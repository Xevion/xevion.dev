use axum::{Json, extract::State, response::IntoResponse};
use std::sync::Arc;

use crate::{
    db,
    state::{AdminSession, AppResult, AppState},
};

#[tracing::instrument(skip_all)]
pub async fn get_settings_handler(
    State(state): State<Arc<AppState>>,
) -> AppResult<impl IntoResponse> {
    let settings = db::get_site_settings(&state.pool).await?;
    Ok(Json(settings))
}

#[tracing::instrument(skip_all)]
pub async fn update_settings_handler(
    State(state): State<Arc<AppState>>,
    _session: AdminSession,
    Json(payload): Json<db::UpdateSiteSettingsRequest>,
) -> AppResult<impl IntoResponse> {
    let settings = db::update_site_settings(&state.pool, &payload).await?;
    tracing::info!("Site settings updated");
    Ok(Json(settings))
}
