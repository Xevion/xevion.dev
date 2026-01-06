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

// Tag database models
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbTag {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub links: Vec<ApiProjectLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTag {
    pub id: String,
    pub slug: String,
    pub name: String,
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
) -> Result<DbTag, sqlx::Error> {
    let slug = slug_override
        .map(|s| slugify(s))
        .unwrap_or_else(|| slugify(name));

    sqlx::query_as!(
        DbTag,
        r#"
        INSERT INTO tags (slug, name)
        VALUES ($1, $2)
        RETURNING id, slug, name, created_at
        "#,
        slug,
        name
    )
    .fetch_one(pool)
    .await
}

pub async fn get_tag_by_id(pool: &PgPool, id: Uuid) -> Result<Option<DbTag>, sqlx::Error> {
    sqlx::query_as!(
        DbTag,
        r#"
        SELECT id, slug, name, created_at
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
        SELECT id, slug, name, created_at
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
        SELECT id, slug, name, created_at
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
            t.created_at,
            COUNT(pt.project_id)::int as "project_count!"
        FROM tags t
        LEFT JOIN project_tags pt ON t.id = pt.tag_id
        GROUP BY t.id, t.slug, t.name, t.created_at
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
) -> Result<DbTag, sqlx::Error> {
    let slug = slug_override
        .map(|s| slugify(s))
        .unwrap_or_else(|| slugify(name));

    sqlx::query_as!(
        DbTag,
        r#"
        UPDATE tags
        SET slug = $2, name = $3
        WHERE id = $1
        RETURNING id, slug, name, created_at
        "#,
        id,
        slug,
        name
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
        SELECT t.id, t.slug, t.name, t.created_at
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
            p.title, 
            p.description, 
            p.status as "status: ProjectStatus", 
            p.github_repo, 
            p.demo_url, 
            p.priority, 
            p.icon, 
            p.last_github_activity, 
            p.created_at, 
            p.updated_at
        FROM projects p
        JOIN project_tags pt ON p.id = pt.project_id
        WHERE pt.tag_id = $1
        ORDER BY p.priority DESC, p.created_at DESC
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
                created_at: row.created_at,
            };
            (tag, row.count)
        })
        .collect())
}
