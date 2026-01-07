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

// Tag database models
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbTag {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbProjectTag {
    pub project_id: Uuid,
    pub tag_id: Uuid,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbTagCooccurrence {
    pub tag_a: Uuid,
    pub tag_b: Uuid,
    pub count: i32,
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
pub struct ApiTag {
    pub id: String,
    pub slug: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiProjectWithTags {
    #[serde(flatten)]
    pub project: ApiProject,
    pub tags: Vec<ApiTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTagWithCount {
    #[serde(flatten)]
    pub tag: ApiTag,
    pub project_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRelatedTag {
    #[serde(flatten)]
    pub tag: ApiTag,
    pub cooccurrence_count: i32,
}

impl DbTag {
    /// Convert database tag to API response format
    pub fn to_api_tag(&self) -> ApiTag {
        ApiTag {
            id: self.id.to_string(),
            slug: self.slug.clone(),
            name: self.name.clone(),
            icon: self.icon.clone(),
            color: self.color.clone(),
        }
    }
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
) -> Result<Vec<(DbProject, Vec<DbTag>)>, sqlx::Error> {
    let projects = get_public_projects(pool).await?;

    let mut result = Vec::new();
    for project in projects {
        let tags = get_tags_for_project(pool, project.id).await?;
        result.push((project, tags));
    }

    Ok(result)
}

pub async fn health_check(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!("SELECT 1 as check")
        .fetch_one(pool)
        .await
        .map(|_| ())
}

// Helper function: slugify text
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

// Tag CRUD queries

pub async fn create_tag(
    pool: &PgPool,
    name: &str,
    slug_override: Option<&str>,
    icon: Option<&str>,
    color: Option<&str>,
) -> Result<DbTag, sqlx::Error> {
    let slug = slug_override
        .map(|s| slugify(s))
        .unwrap_or_else(|| slugify(name));

    sqlx::query_as!(
        DbTag,
        r#"
        INSERT INTO tags (slug, name, icon, color)
        VALUES ($1, $2, $3, $4)
        RETURNING id, slug, name, icon, color, created_at
        "#,
        slug,
        name,
        icon,
        color
    )
    .fetch_one(pool)
    .await
}

pub async fn get_tag_by_id(pool: &PgPool, id: Uuid) -> Result<Option<DbTag>, sqlx::Error> {
    sqlx::query_as!(
        DbTag,
        r#"
        SELECT id, slug, name, icon, color, created_at
        FROM tags
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_tag_by_slug(pool: &PgPool, slug: &str) -> Result<Option<DbTag>, sqlx::Error> {
    sqlx::query_as!(
        DbTag,
        r#"
        SELECT id, slug, name, icon, color, created_at
        FROM tags
        WHERE slug = $1
        "#,
        slug
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_all_tags(pool: &PgPool) -> Result<Vec<DbTag>, sqlx::Error> {
    sqlx::query_as!(
        DbTag,
        r#"
        SELECT id, slug, name, icon, color, created_at
        FROM tags
        ORDER BY name ASC
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn get_all_tags_with_counts(pool: &PgPool) -> Result<Vec<(DbTag, i32)>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT 
            t.id, 
            t.slug, 
            t.name,
            t.icon,
            t.color,
            t.created_at,
            COUNT(pt.project_id)::int as "project_count!"
        FROM tags t
        LEFT JOIN project_tags pt ON t.id = pt.tag_id
        GROUP BY t.id, t.slug, t.name, t.icon, t.color, t.created_at
        ORDER BY t.name ASC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let tag = DbTag {
                id: row.id,
                slug: row.slug,
                name: row.name,
                icon: row.icon,
                color: row.color,
                created_at: row.created_at,
            };
            (tag, row.project_count)
        })
        .collect())
}

pub async fn update_tag(
    pool: &PgPool,
    id: Uuid,
    name: &str,
    slug_override: Option<&str>,
    icon: Option<&str>,
    color: Option<&str>,
) -> Result<DbTag, sqlx::Error> {
    let slug = slug_override
        .map(|s| slugify(s))
        .unwrap_or_else(|| slugify(name));

    sqlx::query_as!(
        DbTag,
        r#"
        UPDATE tags
        SET slug = $2, name = $3, icon = $4, color = $5
        WHERE id = $1
        RETURNING id, slug, name, icon, color, created_at
        "#,
        id,
        slug,
        name,
        icon,
        color
    )
    .fetch_one(pool)
    .await
}

pub async fn delete_tag(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM tags WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn tag_exists_by_name(pool: &PgPool, name: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT EXISTS(SELECT 1 FROM tags WHERE LOWER(name) = LOWER($1)) as "exists!"
        "#,
        name
    )
    .fetch_one(pool)
    .await?;

    Ok(result.exists)
}

pub async fn tag_exists_by_slug(pool: &PgPool, slug: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT EXISTS(SELECT 1 FROM tags WHERE slug = $1) as "exists!"
        "#,
        slug
    )
    .fetch_one(pool)
    .await?;

    Ok(result.exists)
}

// Project-Tag association queries

pub async fn add_tag_to_project(
    pool: &PgPool,
    project_id: Uuid,
    tag_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO project_tags (project_id, tag_id)
        VALUES ($1, $2)
        ON CONFLICT (project_id, tag_id) DO NOTHING
        "#,
        project_id,
        tag_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn remove_tag_from_project(
    pool: &PgPool,
    project_id: Uuid,
    tag_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM project_tags WHERE project_id = $1 AND tag_id = $2",
        project_id,
        tag_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_tags_for_project(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<DbTag>, sqlx::Error> {
    sqlx::query_as!(
        DbTag,
        r#"
        SELECT t.id, t.slug, t.name, t.icon, t.color, t.created_at
        FROM tags t
        JOIN project_tags pt ON t.id = pt.tag_id
        WHERE pt.project_id = $1
        ORDER BY t.name ASC
        "#,
        project_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get_projects_for_tag(
    pool: &PgPool,
    tag_id: Uuid,
) -> Result<Vec<DbProject>, sqlx::Error> {
    sqlx::query_as!(
        DbProject,
        r#"
        SELECT 
            p.id, 
            p.slug, 
            p.name,
            p.short_description,
            p.description, 
            p.status as "status: ProjectStatus", 
            p.github_repo, 
            p.demo_url, 
            p.last_github_activity, 
            p.created_at, 
            p.updated_at
        FROM projects p
        JOIN project_tags pt ON p.id = pt.project_id
        WHERE pt.tag_id = $1
        ORDER BY p.updated_at DESC
        "#,
        tag_id
    )
    .fetch_all(pool)
    .await
}

// Tag cooccurrence queries

pub async fn recalculate_tag_cooccurrence(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Delete existing cooccurrence data
    sqlx::query!("DELETE FROM tag_cooccurrence")
        .execute(pool)
        .await?;

    // Calculate and insert new cooccurrence data
    sqlx::query!(
        r#"
        INSERT INTO tag_cooccurrence (tag_a, tag_b, count)
        SELECT 
            LEAST(t1.tag_id, t2.tag_id) as tag_a,
            GREATEST(t1.tag_id, t2.tag_id) as tag_b,
            COUNT(*)::int as count
        FROM project_tags t1
        JOIN project_tags t2 ON t1.project_id = t2.project_id
        WHERE t1.tag_id < t2.tag_id
        GROUP BY tag_a, tag_b
        HAVING COUNT(*) > 0
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_related_tags(
    pool: &PgPool,
    tag_id: Uuid,
    limit: i64,
) -> Result<Vec<(DbTag, i32)>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT 
            t.id, 
            t.slug, 
            t.name,
            t.icon,
            t.color,
            t.created_at,
            tc.count
        FROM tag_cooccurrence tc
        JOIN tags t ON (tc.tag_a = t.id OR tc.tag_b = t.id)
        WHERE (tc.tag_a = $1 OR tc.tag_b = $1) AND t.id != $1
        ORDER BY tc.count DESC, t.name ASC
        LIMIT $2
        "#,
        tag_id,
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let tag = DbTag {
                id: row.id,
                slug: row.slug,
                name: row.name,
                icon: row.icon,
                color: row.color,
                created_at: row.created_at,
            };
            (tag, row.count)
        })
        .collect())
}

// Project CRUD request/response types

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

// Response type for admin project list/detail (includes tags and metadata)
#[derive(Debug, Clone, Serialize)]
pub struct ApiAdminProject {
    #[serde(flatten)]
    pub project: ApiProject,
    pub tags: Vec<ApiTag>,
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
    pub fn to_api_admin_project(&self, tags: Vec<DbTag>) -> ApiAdminProject {
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

// Project CRUD queries

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
) -> Result<Vec<(DbProject, Vec<DbTag>)>, sqlx::Error> {
    let projects = get_all_projects_admin(pool).await?;

    let mut result = Vec::new();
    for project in projects {
        let tags = get_tags_for_project(pool, project.id).await?;
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
) -> Result<Option<(DbProject, Vec<DbTag>)>, sqlx::Error> {
    let project = get_project_by_id(pool, id).await?;

    match project {
        Some(p) => {
            let tags = get_tags_for_project(pool, p.id).await?;
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

/// Set project tags (smart diff implementation)
pub async fn set_project_tags(
    pool: &PgPool,
    project_id: Uuid,
    tag_ids: &[Uuid],
) -> Result<(), sqlx::Error> {
    // Get current tags
    let current_tags = get_tags_for_project(pool, project_id).await?;
    let current_ids: Vec<Uuid> = current_tags.iter().map(|t| t.id).collect();

    // Find tags to add (in new list but not in current)
    let to_add: Vec<Uuid> = tag_ids
        .iter()
        .filter(|id| !current_ids.contains(id))
        .copied()
        .collect();

    // Find tags to remove (in current but not in new list)
    let to_remove: Vec<Uuid> = current_ids
        .iter()
        .filter(|id| !tag_ids.contains(id))
        .copied()
        .collect();

    // Add new tags
    for tag_id in to_add {
        add_tag_to_project(pool, project_id, tag_id).await?;
    }

    // Remove old tags
    for tag_id in to_remove {
        remove_tag_from_project(pool, project_id, tag_id).await?;
    }

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

// Site settings models and queries

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbSiteIdentity {
    pub id: i32,
    pub display_name: String,
    pub occupation: String,
    pub bio: String,
    pub site_title: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbSocialLink {
    pub id: Uuid,
    pub platform: String,
    pub label: String,
    pub value: String,
    pub icon: String,
    pub visible: bool,
    pub display_order: i32,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

// API response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSiteIdentity {
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub occupation: String,
    pub bio: String,
    #[serde(rename = "siteTitle")]
    pub site_title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSocialLink {
    pub id: String,
    pub platform: String,
    pub label: String,
    pub value: String,
    pub icon: String,
    pub visible: bool,
    #[serde(rename = "displayOrder")]
    pub display_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSiteSettings {
    pub identity: ApiSiteIdentity,
    #[serde(rename = "socialLinks")]
    pub social_links: Vec<ApiSocialLink>,
}

// Request types for updates
#[derive(Debug, Deserialize)]
pub struct UpdateSiteIdentityRequest {
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub occupation: String,
    pub bio: String,
    #[serde(rename = "siteTitle")]
    pub site_title: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSocialLinkRequest {
    pub id: String,
    pub platform: String,
    pub label: String,
    pub value: String,
    pub icon: String,
    pub visible: bool,
    #[serde(rename = "displayOrder")]
    pub display_order: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSiteSettingsRequest {
    pub identity: UpdateSiteIdentityRequest,
    #[serde(rename = "socialLinks")]
    pub social_links: Vec<UpdateSocialLinkRequest>,
}

// Conversion implementations
impl DbSiteIdentity {
    pub fn to_api(&self) -> ApiSiteIdentity {
        ApiSiteIdentity {
            display_name: self.display_name.clone(),
            occupation: self.occupation.clone(),
            bio: self.bio.clone(),
            site_title: self.site_title.clone(),
        }
    }
}

impl DbSocialLink {
    pub fn to_api(&self) -> ApiSocialLink {
        ApiSocialLink {
            id: self.id.to_string(),
            platform: self.platform.clone(),
            label: self.label.clone(),
            value: self.value.clone(),
            icon: self.icon.clone(),
            visible: self.visible,
            display_order: self.display_order,
        }
    }
}

// Query functions
pub async fn get_site_settings(pool: &PgPool) -> Result<ApiSiteSettings, sqlx::Error> {
    // Get identity (single row)
    let identity = sqlx::query_as!(
        DbSiteIdentity,
        r#"
        SELECT id, display_name, occupation, bio, site_title, created_at, updated_at
        FROM site_identity
        WHERE id = 1
        "#
    )
    .fetch_one(pool)
    .await?;

    // Get social links (ordered)
    let social_links = sqlx::query_as!(
        DbSocialLink,
        r#"
        SELECT id, platform, label, value, icon, visible, display_order, created_at, updated_at
        FROM social_links
        ORDER BY display_order ASC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(ApiSiteSettings {
        identity: identity.to_api(),
        social_links: social_links.into_iter().map(|sl| sl.to_api()).collect(),
    })
}

pub async fn update_site_identity(
    pool: &PgPool,
    req: &UpdateSiteIdentityRequest,
) -> Result<DbSiteIdentity, sqlx::Error> {
    sqlx::query_as!(
        DbSiteIdentity,
        r#"
        UPDATE site_identity
        SET display_name = $1, occupation = $2, bio = $3, site_title = $4
        WHERE id = 1
        RETURNING id, display_name, occupation, bio, site_title, created_at, updated_at
        "#,
        req.display_name,
        req.occupation,
        req.bio,
        req.site_title
    )
    .fetch_one(pool)
    .await
}

pub async fn update_social_link(
    pool: &PgPool,
    link_id: Uuid,
    req: &UpdateSocialLinkRequest,
) -> Result<DbSocialLink, sqlx::Error> {
    sqlx::query_as!(
        DbSocialLink,
        r#"
        UPDATE social_links
        SET platform = $2, label = $3, value = $4, icon = $5, visible = $6, display_order = $7
        WHERE id = $1
        RETURNING id, platform, label, value, icon, visible, display_order, created_at, updated_at
        "#,
        link_id,
        req.platform,
        req.label,
        req.value,
        req.icon,
        req.visible,
        req.display_order
    )
    .fetch_one(pool)
    .await
}

pub async fn update_site_settings(
    pool: &PgPool,
    req: &UpdateSiteSettingsRequest,
) -> Result<ApiSiteSettings, sqlx::Error> {
    // Update identity
    let identity = update_site_identity(pool, &req.identity).await?;

    // Update each social link
    let mut updated_links = Vec::new();
    for link_req in &req.social_links {
        let link_id = Uuid::parse_str(&link_req.id).map_err(|_| {
            sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid UUID format",
            )))
        })?;
        let link = update_social_link(pool, link_id, link_req).await?;
        updated_links.push(link);
    }

    Ok(ApiSiteSettings {
        identity: identity.to_api(),
        social_links: updated_links.into_iter().map(|sl| sl.to_api()).collect(),
    })
}
