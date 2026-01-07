pub mod projects;
pub mod settings;
pub mod tags;

// Re-export all types and functions
pub use projects::*;
pub use settings::*;
pub use tags::*;

use sqlx::{PgPool, postgres::PgPoolOptions, query};

/// Database connection pool creation
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(database_url)
        .await
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
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '\0'
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
pub enum ProjectStatus {
    Active,
    Maintained,
    Archived,
    Hidden,
}
