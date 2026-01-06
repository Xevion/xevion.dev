use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use time::OffsetDateTime;
use uuid::Uuid;

// Database types
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "project_status", rename_all = "lowercase")]
pub enum ProjectStatus {
    Active,
    Maintained,
    Archived,
    Hidden,
}

// Database model
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct DbProject {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub status: ProjectStatus,
    pub github_repo: Option<String>,
    pub demo_url: Option<String>,
    pub priority: i32,
    pub icon: Option<String>,
    pub last_github_activity: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

// API response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiProjectLink {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiProject {
    pub id: String,
    pub name: String,
    #[serde(rename = "shortDescription")]
    pub short_description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub links: Vec<ApiProjectLink>,
}

impl DbProject {
    /// Convert database project to API response format
    pub fn to_api_project(&self) -> ApiProject {
        let mut links = Vec::new();

        if let Some(ref repo) = self.github_repo {
            links.push(ApiProjectLink {
                url: format!("https://github.com/{}", repo),
                title: Some("GitHub".to_string()),
            });
        }

        if let Some(ref demo) = self.demo_url {
            links.push(ApiProjectLink {
                url: demo.clone(),
                title: Some("Demo".to_string()),
            });
        }

        ApiProject {
            id: self.id.to_string(),
            name: self.title.clone(),
            short_description: self.description.clone(),
            icon: self.icon.clone(),
            links,
        }
    }
}

// Connection pool creation
pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(std::time::Duration::from_secs(3))
        .connect(database_url)
        .await
}

// Queries
pub async fn get_public_projects(pool: &PgPool) -> Result<Vec<DbProject>, sqlx::Error> {
    sqlx::query_as!(
        DbProject,
        r#"
        SELECT 
            id, 
            slug, 
            title, 
            description, 
            status as "status: ProjectStatus", 
            github_repo, 
            demo_url, 
            priority, 
            icon, 
            last_github_activity, 
            created_at, 
            updated_at
        FROM projects
        WHERE status != 'hidden'
        ORDER BY priority DESC, created_at DESC
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn health_check(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!("SELECT 1 as check")
        .fetch_one(pool)
        .await
        .map(|_| ())
}
