use std::sync::Arc;

use crate::{
    auth::SessionManager, cache::IsrCache, health::HealthChecker, http::HttpClient,
    icon_cache::IconCache, tarpit::TarpitState,
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
    pub icon_cache: Arc<IconCache>,
}

/// Errors that can occur during proxying to Bun
#[derive(Debug)]
pub enum ProxyError {
    Network(reqwest::Error),
}

impl std::fmt::Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyError::Network(e) => write!(f, "Network error: {e}"),
        }
    }
}

impl std::error::Error for ProxyError {}

/// Typed API errors with automatic HTTP response mapping
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found")]
    NotFound,

    #[error("Authentication required")]
    Unauthorized,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("{0}")]
    Validation(String),

    #[error("{0}")]
    Conflict(String),

    #[error("{0}")]
    ServiceUnavailable(String),

    #[error(transparent)]
    Database(sqlx::Error),

    #[error("{0}")]
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        use axum::{Json, http::StatusCode};

        let (status, code) = match &self {
            Self::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            Self::InvalidCredentials => (StatusCode::UNAUTHORIZED, "INVALID_CREDENTIALS"),
            Self::Validation(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR"),
            Self::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT"),
            Self::ServiceUnavailable(_) => (StatusCode::SERVICE_UNAVAILABLE, "SERVICE_UNAVAILABLE"),
            Self::Database(err) => {
                tracing::error!(error = %err, "Database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR")
            }
            Self::Internal(err) => {
                tracing::error!(error = %err, "Internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR")
            }
        };

        let message = match &self {
            Self::Database(_) | Self::Internal(_) => "Internal server error".to_string(),
            other => other.to_string(),
        };

        (
            status,
            Json(serde_json::json!({ "error": message, "code": code })),
        )
            .into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err)
    }
}

/// Convert `Option<T>` to `Result<T, AppError::NotFound>`
pub trait OptionNotFoundExt<T> {
    fn or_not_found(self) -> Result<T, AppError>;
}

impl<T> OptionNotFoundExt<T> for Option<T> {
    fn or_not_found(self) -> Result<T, AppError> {
        self.ok_or(AppError::NotFound)
    }
}

/// Map specific database errors to domain errors
pub trait SqlxResultExt<T> {
    fn conflict_on_unique(self, msg: impl Into<String>) -> Result<T, AppError>;
    fn not_found_on_fk(self) -> Result<T, AppError>;
}

impl<T> SqlxResultExt<T> for Result<T, sqlx::Error> {
    fn conflict_on_unique(self, msg: impl Into<String>) -> Result<T, AppError> {
        self.map_err(|err| match &err {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                AppError::Conflict(msg.into())
            }
            _ => AppError::Database(err),
        })
    }

    fn not_found_on_fk(self) -> Result<T, AppError> {
        self.map_err(|err| match &err {
            sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
                AppError::NotFound
            }
            _ => AppError::Database(err),
        })
    }
}

/// Auth extractor — validates the admin session from cookies.
/// Use in handler signatures to require authentication.
#[derive(Debug)]
pub struct AdminSession(pub crate::auth::Session);

impl axum::extract::FromRequestParts<Arc<AppState>> for AdminSession {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let jar = axum_extra::extract::CookieJar::from_headers(&parts.headers);
        let session = crate::auth::check_session(state, &jar).ok_or(AppError::Unauthorized)?;
        Ok(AdminSession(session))
    }
}
