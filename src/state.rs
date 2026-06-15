use std::collections::BTreeMap;
use std::sync::Arc;

use crate::{
    auth::SessionManager, cache::IsrCache, cli_auth::CliAuthRegistry, events::EventSender,
    health::HealthChecker, http::HttpClient, icon_cache::IconCache, tarpit::TarpitState,
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
    pub event_sender: EventSender,
    pub cli_auth: CliAuthRegistry,
}

/// Errors that can occur during proxying to Bun
#[derive(Debug)]
pub enum ProxyError {
    Network(reqwest::Error),
}

impl std::fmt::Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(e) => write!(f, "Network error: {e}"),
        }
    }
}

impl std::error::Error for ProxyError {}

/// Structured validation errors with optional per-field messages.
///
/// When `fields` is non-empty, each key is a camelCase JSON request field name.
/// When `fields` is empty, `general` carries the error message.
#[derive(Debug, Default)]
pub struct ValidationErrors {
    /// Per-field error messages keyed by camelCase JSON field name.
    pub fields: BTreeMap<String, String>,
    /// General (non-field-specific) error message used when `fields` is empty.
    pub general: Option<String>,
}

impl ValidationErrors {
    /// A single-field validation error.
    pub fn field(field: impl Into<String>, msg: impl Into<String>) -> Self {
        let mut fields = BTreeMap::new();
        fields.insert(field.into(), msg.into());
        Self {
            fields,
            general: None,
        }
    }

    /// A general (non-field-specific) validation error.
    pub fn general(msg: impl Into<String>) -> Self {
        Self {
            fields: BTreeMap::new(),
            general: Some(msg.into()),
        }
    }

    /// Human-readable summary for use in the `error` JSON field and `Display`.
    fn summary(&self) -> String {
        if self.fields.is_empty() {
            self.general
                .clone()
                .unwrap_or_else(|| "Validation failed".to_string())
        } else {
            "Validation failed".to_string()
        }
    }
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.summary())
    }
}

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
    Validation(ValidationErrors),

    #[error("{0}")]
    Conflict(String),

    #[error("{0}")]
    ServiceUnavailable(String),

    #[error(transparent)]
    Database(sqlx::Error),

    #[error("{0}")]
    Internal(String),
}

impl AppError {
    /// Validation error for a single named request field.
    pub fn field(field: impl Into<String>, msg: impl Into<String>) -> Self {
        Self::Validation(ValidationErrors::field(field, msg))
    }

    /// General (non-field-specific) validation error.
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(ValidationErrors::general(msg))
    }
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

        if let Self::Validation(ref errs) = self {
            return (
                status,
                Json(serde_json::json!({
                    "error": message,
                    "code": code,
                    "fieldErrors": errs.fields,
                })),
            )
                .into_response();
        }

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

/// Auth extractor — validates the admin session from either the browser cookie
/// or a CLI `Authorization: Bearer` token. Use in handler signatures to require
/// authentication.
#[derive(Debug)]
pub struct AdminSession(pub crate::auth::Session);

impl axum::extract::FromRequestParts<Arc<AppState>> for AdminSession {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let session = crate::auth::authenticate(state, &parts.headers)
            .await
            .ok_or(AppError::Unauthorized)?;
        Ok(Self(session))
    }
}
