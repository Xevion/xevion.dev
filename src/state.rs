use std::sync::Arc;

use crate::{
    auth::SessionManager, cache::IsrCache, health::HealthChecker, http::HttpClient,
    tarpit::TarpitState,
};

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub client: HttpClient,
    pub health_checker: Arc<HealthChecker>,
    pub tarpit_state: Arc<TarpitState>,
    pub pool: sqlx::PgPool,
    pub session_manager: Arc<SessionManager>,
    pub isr_cache: Arc<IsrCache>,
}

/// Errors that can occur during proxying to Bun
#[derive(Debug)]
pub enum ProxyError {
    Network(reqwest::Error),
    Other(String),
}

impl std::fmt::Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyError::Network(e) => write!(f, "Network error: {e}"),
            ProxyError::Other(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for ProxyError {}
