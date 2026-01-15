use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::slugify;

// Tag database models
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbTag {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
}

// API response types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct ApiTagWithCount {
    #[serde(flatten)]
    pub tag: ApiTag,
    pub project_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

// Tag CRUD queries

pub async fn create_tag(
    pool: &PgPool,
    name: &str,
    slug_override: Option<&str>,
    icon: Option<&str>,
    color: Option<&str>,
) -> Result<DbTag, sqlx::Error> {
    let slug = slug_override.map(slugify).unwrap_or_else(|| slugify(name));

    sqlx::query_as!(
        DbTag,
        r#"
        INSERT INTO tags (slug, name, icon, color)
        VALUES ($1, $2, $3, $4)
        RETURNING id, slug, name, icon, color
        "#,
        slug,
        name,
        icon,
        color
    )
    .fetch_one(pool)
    .await
}

pub async fn get_tag_by_slug(pool: &PgPool, slug: &str) -> Result<Option<DbTag>, sqlx::Error> {
    sqlx::query_as!(
        DbTag,
        r#"
        SELECT id, slug, name, icon, color
        FROM tags
        WHERE slug = $1
        "#,
        slug
    )
    .fetch_optional(pool)
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
            COUNT(pt.project_id)::int as "project_count!"
        FROM tags t
        LEFT JOIN project_tags pt ON t.id = pt.tag_id
        GROUP BY t.id, t.slug, t.name, t.icon, t.color
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
    let slug = slug_override.map(slugify).unwrap_or_else(|| slugify(name));

    sqlx::query_as!(
        DbTag,
        r#"
        UPDATE tags
        SET slug = $2, name = $3, icon = $4, color = $5
        WHERE id = $1
        RETURNING id, slug, name, icon, color
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
        SELECT t.id, t.slug, t.name, t.icon, t.color
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
) -> Result<Vec<super::projects::DbProject>, sqlx::Error> {
    sqlx::query_as!(
        super::projects::DbProject,
        r#"
        SELECT 
            p.id, 
            p.slug, 
            p.name,
            p.short_description,
            p.description, 
            p.status as "status: super::ProjectStatus", 
            p.github_repo, 
            p.demo_url, 
            p.last_github_activity, 
            p.created_at
        FROM projects p
        JOIN project_tags pt ON p.id = pt.project_id
        WHERE pt.tag_id = $1
        ORDER BY COALESCE(p.last_github_activity, p.created_at) DESC
        "#,
        tag_id
    )
    .fetch_all(pool)
    .await
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
            };
            (tag, row.count)
        })
        .collect())
}
