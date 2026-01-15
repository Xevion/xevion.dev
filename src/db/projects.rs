use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, query, query_as};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use uuid::Uuid;

use super::{
    ProjectStatus,
    media::{self, ApiProjectMedia, DbProjectMedia, get_media_for_project},
    slugify,
    tags::{self, ApiTag, DbTag, get_tags_for_project},
};

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
}

// API response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiProjectLink {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiProject {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub short_description: String,
    pub links: Vec<ApiProjectLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiAdminProject {
    #[serde(flatten)]
    pub project: ApiProject,
    pub tags: Vec<ApiTag>,
    pub media: Vec<ApiProjectMedia>,
    pub status: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demo_url: Option<String>,
    pub created_at: String,    // ISO 8601
    pub last_activity: String, // ISO 8601
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

    pub fn to_api_admin_project(
        &self,
        tags: Vec<DbTag>,
        media: Vec<DbProjectMedia>,
    ) -> ApiAdminProject {
        let last_activity = self
            .last_github_activity
            .unwrap_or(self.created_at)
            .format(&Rfc3339)
            .unwrap();

        ApiAdminProject {
            project: self.to_api_project(),
            tags: tags.into_iter().map(|t| t.to_api_tag()).collect(),
            media: media.into_iter().map(|m| m.to_api_media()).collect(),
            status: format!("{:?}", self.status).to_lowercase(),
            description: self.description.clone(),
            github_repo: self.github_repo.clone(),
            demo_url: self.demo_url.clone(),
            created_at: self.created_at.format(&Rfc3339).unwrap(),
            last_activity,
        }
    }
}

// Request types for CRUD operations

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct AdminStats {
    pub total_projects: i32,
    pub projects_by_status: serde_json::Value,
    pub total_tags: i32,
}

// Query functions

pub async fn get_public_projects(pool: &PgPool) -> Result<Vec<DbProject>, sqlx::Error> {
    query_as!(
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
            created_at
        FROM projects
        WHERE status != 'hidden'
        ORDER BY COALESCE(last_github_activity, created_at) DESC
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn get_public_projects_with_tags(
    pool: &PgPool,
) -> Result<Vec<(DbProject, Vec<DbTag>, Vec<DbProjectMedia>)>, sqlx::Error> {
    let projects = get_public_projects(pool).await?;

    if projects.is_empty() {
        return Ok(Vec::new());
    }

    // Collect project IDs for batch queries
    let project_ids: Vec<Uuid> = projects.iter().map(|p| p.id).collect();

    // Batch fetch all tags and media in 2 queries instead of N*2
    let (tags_map, media_map) = tokio::try_join!(
        tags::get_tags_for_projects(pool, &project_ids),
        media::get_media_for_projects(pool, &project_ids),
    )?;

    // Assemble results
    let result = projects
        .into_iter()
        .map(|project| {
            let tags = tags_map.get(&project.id).cloned().unwrap_or_default();
            let media = media_map.get(&project.id).cloned().unwrap_or_default();
            (project, tags, media)
        })
        .collect();

    Ok(result)
}

/// Get all projects (admin view - includes hidden)
pub async fn get_all_projects_admin(pool: &PgPool) -> Result<Vec<DbProject>, sqlx::Error> {
    query_as!(
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
            created_at
        FROM projects
        ORDER BY COALESCE(last_github_activity, created_at) DESC
        "#
    )
    .fetch_all(pool)
    .await
}

/// Get all projects with tags and media (admin view)
pub async fn get_all_projects_with_tags_admin(
    pool: &PgPool,
) -> Result<Vec<(DbProject, Vec<DbTag>, Vec<DbProjectMedia>)>, sqlx::Error> {
    let projects = get_all_projects_admin(pool).await?;

    if projects.is_empty() {
        return Ok(Vec::new());
    }

    // Collect project IDs for batch queries
    let project_ids: Vec<Uuid> = projects.iter().map(|p| p.id).collect();

    // Batch fetch all tags and media in 2 queries instead of N*2
    let (tags_map, media_map) = tokio::try_join!(
        tags::get_tags_for_projects(pool, &project_ids),
        media::get_media_for_projects(pool, &project_ids),
    )?;

    // Assemble results
    let result = projects
        .into_iter()
        .map(|project| {
            let tags = tags_map.get(&project.id).cloned().unwrap_or_default();
            let media = media_map.get(&project.id).cloned().unwrap_or_default();
            (project, tags, media)
        })
        .collect();

    Ok(result)
}

/// Get single project by ID
pub async fn get_project_by_id(pool: &PgPool, id: Uuid) -> Result<Option<DbProject>, sqlx::Error> {
    query_as!(
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
            created_at
        FROM projects
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
}

/// Get single project by ID with tags and media
pub async fn get_project_by_id_with_tags(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<(DbProject, Vec<DbTag>, Vec<DbProjectMedia>)>, sqlx::Error> {
    let project = get_project_by_id(pool, id).await?;

    match project {
        Some(p) => {
            let tags = get_tags_for_project(pool, p.id).await?;
            let media = get_media_for_project(pool, p.id).await?;
            Ok(Some((p, tags, media)))
        }
        None => Ok(None),
    }
}

/// Get single project by slug
pub async fn get_project_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<DbProject>, sqlx::Error> {
    query_as!(
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
            created_at
        FROM projects
        WHERE slug = $1
        "#,
        slug
    )
    .fetch_optional(pool)
    .await
}

/// Get a project by either UUID or slug (auto-detects format)
pub async fn get_project_by_ref(
    pool: &PgPool,
    ref_str: &str,
) -> Result<Option<DbProject>, sqlx::Error> {
    if let Ok(uuid) = Uuid::parse_str(ref_str) {
        get_project_by_id(pool, uuid).await
    } else {
        get_project_by_slug(pool, ref_str).await
    }
}

/// Get a project by ref (UUID or slug) with tags and media
pub async fn get_project_by_ref_with_tags(
    pool: &PgPool,
    ref_str: &str,
) -> Result<Option<(DbProject, Vec<DbTag>, Vec<DbProjectMedia>)>, sqlx::Error> {
    let project = get_project_by_ref(pool, ref_str).await?;

    match project {
        Some(p) => {
            let tags = get_tags_for_project(pool, p.id).await?;
            let media = get_media_for_project(pool, p.id).await?;
            Ok(Some((p, tags, media)))
        }
        None => Ok(None),
    }
}

/// Create project (without tags - tags handled separately)
#[allow(clippy::too_many_arguments)]
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
    let slug = slug_override.map(slugify).unwrap_or_else(|| slugify(name));

    query_as!(
        DbProject,
        r#"
        INSERT INTO projects (slug, name, short_description, description, status, github_repo, demo_url)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, slug, name, short_description, description, status as "status: ProjectStatus",
                  github_repo, demo_url, last_github_activity, created_at
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
#[allow(clippy::too_many_arguments)]
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
    let slug = slug_override.map(slugify).unwrap_or_else(|| slugify(name));

    query_as!(
        DbProject,
        r#"
        UPDATE projects
        SET slug = $2, name = $3, short_description = $4, description = $5,
            status = $6, github_repo = $7, demo_url = $8
        WHERE id = $1
        RETURNING id, slug, name, short_description, description, status as "status: ProjectStatus",
                  github_repo, demo_url, last_github_activity, created_at
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
    query!("DELETE FROM projects WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Get admin stats
pub async fn get_admin_stats(pool: &PgPool) -> Result<AdminStats, sqlx::Error> {
    // Get project counts by status
    let status_counts = query!(
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

    let mut projects_by_status = json!({
        "active": 0,
        "maintained": 0,
        "archived": 0,
        "hidden": 0,
    });

    let mut total_projects = 0;
    for row in status_counts {
        let status_str = format!("{:?}", row.status).to_lowercase();
        projects_by_status[status_str] = json!(row.count);
        total_projects += row.count;
    }

    // Get total tags
    let tag_count = query!("SELECT COUNT(*)::int as \"count!\" FROM tags")
        .fetch_one(pool)
        .await?;

    Ok(AdminStats {
        total_projects,
        projects_by_status,
        total_tags: tag_count.count,
    })
}

/// Get all projects that have a github_repo set (for GitHub sync).
///
/// Orders by most recent activity first (NULLS LAST) so that projects with
/// the shortest check intervals are processed first by the scheduler.
pub async fn get_projects_with_github_repo(pool: &PgPool) -> Result<Vec<DbProject>, sqlx::Error> {
    query_as!(
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
            created_at
        FROM projects
        WHERE github_repo IS NOT NULL
        ORDER BY last_github_activity DESC NULLS LAST
        "#
    )
    .fetch_all(pool)
    .await
}

/// Update the last_github_activity timestamp for a project
pub async fn update_last_github_activity(
    pool: &PgPool,
    id: Uuid,
    activity_time: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    query!(
        "UPDATE projects SET last_github_activity = $2 WHERE id = $1",
        id,
        activity_time
    )
    .execute(pool)
    .await?;
    Ok(())
}
