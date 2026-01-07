use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use super::{ProjectStatus, slugify};

// Database model
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbProject {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub short_description: String,
    pub description: String,
    pub status: ProjectStatus,
    pub github_repo: Option<String>,
    pub demo_url: Option<String>,
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
    pub slug: String,
    pub name: String,
    #[serde(rename = "shortDescription")]
    pub short_description: String,
    pub links: Vec<ApiProjectLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAdminProject {
    #[serde(flatten)]
    pub project: ApiProject,
    pub tags: Vec<super::tags::ApiTag>,
    pub status: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demo_url: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String, // ISO 8601
    #[serde(rename = "updatedAt")]
    pub updated_at: String, // ISO 8601
    #[serde(rename = "lastGithubActivity", skip_serializing_if = "Option::is_none")]
    pub last_github_activity: Option<String>, // ISO 8601
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
            slug: self.slug.clone(),
            name: self.name.clone(),
            short_description: self.short_description.clone(),
            links,
        }
    }

    pub fn to_api_admin_project(&self, tags: Vec<super::tags::DbTag>) -> ApiAdminProject {
        ApiAdminProject {
            project: self.to_api_project(),
            tags: tags.into_iter().map(|t| t.to_api_tag()).collect(),
            status: format!("{:?}", self.status).to_lowercase(),
            description: self.description.clone(),
            github_repo: self.github_repo.clone(),
            demo_url: self.demo_url.clone(),
            created_at: self
                .created_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap(),
            updated_at: self
                .updated_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap(),
            last_github_activity: self.last_github_activity.map(|dt| {
                dt.format(&time::format_description::well_known::Rfc3339)
                    .unwrap()
            }),
        }
    }
}

// Request types for CRUD operations

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub slug: Option<String>,
    pub short_description: String,
    pub description: String,
    pub status: ProjectStatus,
    pub github_repo: Option<String>,
    pub demo_url: Option<String>,
    pub tag_ids: Vec<String>, // UUID strings
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: String,
    pub slug: Option<String>,
    pub short_description: String,
    pub description: String,
    pub status: ProjectStatus,
    pub github_repo: Option<String>,
    pub demo_url: Option<String>,
    pub tag_ids: Vec<String>, // UUID strings
}

// Admin stats response
#[derive(Debug, Serialize)]
pub struct AdminStats {
    #[serde(rename = "totalProjects")]
    pub total_projects: i32,
    #[serde(rename = "projectsByStatus")]
    pub projects_by_status: serde_json::Value,
    #[serde(rename = "totalTags")]
    pub total_tags: i32,
}

// Query functions

pub async fn get_public_projects(pool: &PgPool) -> Result<Vec<DbProject>, sqlx::Error> {
    sqlx::query_as!(
        DbProject,
        r#"
        SELECT 
            id, 
            slug, 
            name,
            short_description,
            description, 
            status as "status: ProjectStatus", 
            github_repo, 
            demo_url, 
            last_github_activity, 
            created_at, 
            updated_at
        FROM projects
        WHERE status != 'hidden'
        ORDER BY updated_at DESC
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn get_public_projects_with_tags(
    pool: &PgPool,
) -> Result<Vec<(DbProject, Vec<super::tags::DbTag>)>, sqlx::Error> {
    let projects = get_public_projects(pool).await?;

    let mut result = Vec::new();
    for project in projects {
        let tags = super::tags::get_tags_for_project(pool, project.id).await?;
        result.push((project, tags));
    }

    Ok(result)
}

/// Get all projects (admin view - includes hidden)
pub async fn get_all_projects_admin(pool: &PgPool) -> Result<Vec<DbProject>, sqlx::Error> {
    sqlx::query_as!(
        DbProject,
        r#"
        SELECT 
            id, 
            slug, 
            name,
            short_description,
            description, 
            status as "status: ProjectStatus", 
            github_repo, 
            demo_url, 
            last_github_activity, 
            created_at, 
            updated_at
        FROM projects
        ORDER BY updated_at DESC
        "#
    )
    .fetch_all(pool)
    .await
}

/// Get all projects with tags (admin view)
pub async fn get_all_projects_with_tags_admin(
    pool: &PgPool,
) -> Result<Vec<(DbProject, Vec<super::tags::DbTag>)>, sqlx::Error> {
    let projects = get_all_projects_admin(pool).await?;

    let mut result = Vec::new();
    for project in projects {
        let tags = super::tags::get_tags_for_project(pool, project.id).await?;
        result.push((project, tags));
    }

    Ok(result)
}

/// Get single project by ID
pub async fn get_project_by_id(pool: &PgPool, id: Uuid) -> Result<Option<DbProject>, sqlx::Error> {
    sqlx::query_as!(
        DbProject,
        r#"
        SELECT 
            id, 
            slug, 
            name,
            short_description,
            description, 
            status as "status: ProjectStatus", 
            github_repo, 
            demo_url, 

            last_github_activity, 
            created_at, 
            updated_at
        FROM projects
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
}

/// Get single project by ID with tags
pub async fn get_project_by_id_with_tags(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<(DbProject, Vec<super::tags::DbTag>)>, sqlx::Error> {
    let project = get_project_by_id(pool, id).await?;

    match project {
        Some(p) => {
            let tags = super::tags::get_tags_for_project(pool, p.id).await?;
            Ok(Some((p, tags)))
        }
        None => Ok(None),
    }
}

/// Get single project by slug
pub async fn get_project_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<DbProject>, sqlx::Error> {
    sqlx::query_as!(
        DbProject,
        r#"
        SELECT 
            id, 
            slug, 
            name,
            short_description,
            description, 
            status as "status: ProjectStatus", 
            github_repo, 
            demo_url, 

            last_github_activity, 
            created_at, 
            updated_at
        FROM projects
        WHERE slug = $1
        "#,
        slug
    )
    .fetch_optional(pool)
    .await
}

/// Create project (without tags - tags handled separately)
pub async fn create_project(
    pool: &PgPool,
    name: &str,
    slug_override: Option<&str>,
    short_description: &str,
    description: &str,
    status: ProjectStatus,
    github_repo: Option<&str>,
    demo_url: Option<&str>,
) -> Result<DbProject, sqlx::Error> {
    let slug = slug_override
        .map(|s| slugify(s))
        .unwrap_or_else(|| slugify(name));

    sqlx::query_as!(
        DbProject,
        r#"
        INSERT INTO projects (slug, name, short_description, description, status, github_repo, demo_url)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, slug, name, short_description, description, status as "status: ProjectStatus", 
                  github_repo, demo_url, last_github_activity, created_at, updated_at
        "#,
        slug,
        name,
        short_description,
        description,
        status as ProjectStatus,
        github_repo,
        demo_url
    )
    .fetch_one(pool)
    .await
}

/// Update project (without tags - tags handled separately)
pub async fn update_project(
    pool: &PgPool,
    id: Uuid,
    name: &str,
    slug_override: Option<&str>,
    short_description: &str,
    description: &str,
    status: ProjectStatus,
    github_repo: Option<&str>,
    demo_url: Option<&str>,
) -> Result<DbProject, sqlx::Error> {
    let slug = slug_override
        .map(|s| slugify(s))
        .unwrap_or_else(|| slugify(name));

    sqlx::query_as!(
        DbProject,
        r#"
        UPDATE projects
        SET slug = $2, name = $3, short_description = $4, description = $5, 
            status = $6, github_repo = $7, demo_url = $8
        WHERE id = $1
        RETURNING id, slug, name, short_description, description, status as "status: ProjectStatus", 
                  github_repo, demo_url, last_github_activity, created_at, updated_at
        "#,
        id,
        slug,
        name,
        short_description,
        description,
        status as ProjectStatus,
        github_repo,
        demo_url
    )
    .fetch_one(pool)
    .await
}

/// Delete project (CASCADE will handle tags)
pub async fn delete_project(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM projects WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Get admin stats
pub async fn get_admin_stats(pool: &PgPool) -> Result<AdminStats, sqlx::Error> {
    // Get project counts by status
    let status_counts = sqlx::query!(
        r#"
        SELECT 
            status as "status!: ProjectStatus",
            COUNT(*)::int as "count!"
        FROM projects
        GROUP BY status
        "#
    )
    .fetch_all(pool)
    .await?;

    let mut projects_by_status = serde_json::json!({
        "active": 0,
        "maintained": 0,
        "archived": 0,
        "hidden": 0,
    });

    let mut total_projects = 0;
    for row in status_counts {
        let status_str = format!("{:?}", row.status).to_lowercase();
        projects_by_status[status_str] = serde_json::json!(row.count);
        total_projects += row.count;
    }

    // Get total tags
    let tag_count = sqlx::query!("SELECT COUNT(*)::int as \"count!\" FROM tags")
        .fetch_one(pool)
        .await?;

    Ok(AdminStats {
        total_projects,
        projects_by_status,
        total_tags: tag_count.count,
    })
}
