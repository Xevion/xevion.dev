use axum::{Json, extract::State, response::IntoResponse};
use std::sync::Arc;

use crate::{
    db,
    events::EventLevel,
    state::{AdminSession, AppResult, AppState},
};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListEventsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub level: Option<EventLevel>,
    pub entity_type: Option<String>,
    pub event_type: Option<String>,
}

#[tracing::instrument(skip_all)]
pub async fn list_events_handler(
    State(state): State<Arc<AppState>>,
    _session: AdminSession,
    axum::extract::Query(params): axum::extract::Query<ListEventsQuery>,
) -> AppResult<impl IntoResponse> {
    let filters = db::events::EventFilters {
        limit: params.limit.unwrap_or(100).min(500),
        offset: params.offset.unwrap_or(0).max(0),
        level: params.level,
        entity_type: params.entity_type,
        event_type: params.event_type,
    };

    let events = db::events::get_events(&state.pool, filters).await?;
    Ok(Json(events))
}
