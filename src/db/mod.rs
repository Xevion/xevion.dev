pub mod media;
pub mod projects;
pub mod settings;
pub mod tags;

// Re-export all types and functions
pub use media::*;
pub use projects::*;
pub use settings::*;
pub use tags::*;

use sqlx::{PgPool, postgres::PgPoolOptions, query};
use std::time::Duration;
use tokio::time::sleep;

/// Database connection pool creation with retry logic
///
/// Production: Exponential backoff (1s -> 2s -> 4s... -> 30s cap), max 10 attempts
/// Development: Fail fast (1 attempt)
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let max_attempts: u32 = if cfg!(debug_assertions) { 1 } else { 10 };
    let initial_delay = Duration::from_secs(1);
    let max_delay = Duration::from_secs(30);

    let pool_options = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(10));

    let mut last_error = None;
    let mut delay = initial_delay;

    for attempt in 1..=max_attempts {
        match pool_options.clone().connect(database_url).await {
            Ok(pool) => {
                if attempt > 1 {
                    tracing::info!(attempt, "Database connection established after retry");
                }
                return Ok(pool);
            }
            Err(e) => {
                if attempt < max_attempts {
                    tracing::warn!(
                        attempt,
                        max_attempts,
                        delay_secs = delay.as_secs(),
                        error = %e,
                        "Database connection failed, retrying..."
                    );
                    sleep(delay).await;
                    delay = (delay * 2).min(max_delay);
                }
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap())
}

/// Health check query
pub async fn health_check(pool: &PgPool) -> Result<(), sqlx::Error> {
    query!("SELECT 1 as check")
        .fetch_one(pool)
        .await
        .map(|_| ())
}

/// Slugify text for URL-safe identifiers
pub fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() {
                Some(c)
            } else if c.is_whitespace() || c == '-' || c == '_' || c == '.' {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Project status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "project_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    Active,
    Maintained,
    Archived,
    Hidden,
}
