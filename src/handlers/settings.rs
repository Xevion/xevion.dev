use axum::{Json, extract::State, response::IntoResponse};
use std::sync::Arc;

use crate::{
    auth, db,
    state::{AppError, AppResult, AppState},
};

pub async fn get_settings_handler(
    State(state): State<Arc<AppState>>,
) -> AppResult<impl IntoResponse> {
    let settings = db::get_site_settings(&state.pool).await?;
    Ok(Json(settings))
}

pub async fn update_settings_handler(
    State(state): State<Arc<AppState>>,
    jar: axum_extra::extract::CookieJar,
    Json(payload): Json<db::UpdateSiteSettingsRequest>,
) -> AppResult<impl IntoResponse> {
    auth::check_session(&state, &jar).ok_or(AppError::Unauthorized)?;

    let settings = db::update_site_settings(&state.pool, &payload).await?;
    tracing::info!("Site settings updated");
    Ok(Json(settings))
}
