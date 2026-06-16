use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{PgPool, query, query_as};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use ts_rs::TS;
use uuid::Uuid;

use crate::state::{AppError, AppResult};

use super::{
    ProjectStatus,
    media::{self, ApiProjectMedia, DbProjectMedia, get_media_for_project},
    slugify,
    tags::{self, ApiTag, DbTag, get_tags_for_project},
};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbProject {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub short_description: String,
    pub status: ProjectStatus,
    pub github_repo: Option<String>,
    pub demo_url: Option<String>,
    pub last_github_activity: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    /// Last authored edit; the DB trigger excludes background GitHub syncs.
    pub updated_at: OffsetDateTime,
    pub detail_content: Option<serde_json::Value>,
    /// Authored primary label ("CLI Tool", "Web App", …). Replaces the old
    /// tag-derived "Language" field.
    pub project_type: Option<String>,
    /// Closed-source flag, orthogonal to `status`: a project can be `active`
    /// and still have no public repo.
    pub source_closed: bool,
    /// Optional asciinema-style cast for the CLI hero, shaped like [`TerminalCast`].
    pub terminal_cast: Option<serde_json::Value>,
    /// Explicit per-project accent (hex); the frontend falls back to `#71717a`.
    pub accent_color: Option<String>,
    /// Timestamp of the last successful GitHub poll (distinct from
    /// `last_github_activity`); `None` until the repo has been synced once.
    pub github_synced_at: Option<OffsetDateTime>,
    /// Most recent GitHub sync failure, cleared on success. `None` means healthy
    /// (or never synced); `Some` flags a repo whose sync is broken.
    pub github_sync_error: Option<String>,
}

/// One line of a [`TerminalCast`], tagged by how it should be styled.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum TerminalLineKind {
    /// A typed command (rendered after the accent-colored prompt).
    Cmd,
    /// Standard output.
    Out,
    /// Error output.
    Err,
    /// De-emphasized output (e.g. trailing summary lines).
    Muted,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TerminalLine {
    pub t: TerminalLineKind,
    pub text: String,
}

/// Authored CLI-hero transcript. Stored as JSONB on the project; absent for
/// non-CLI projects (the page then falls through to its normal body).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TerminalCast {
    pub prompt: String,
    pub lines: Vec<TerminalLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ApiProjectLink {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ApiProject {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub short_description: String,
    pub links: Vec<ApiProjectLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ApiAdminProject {
    #[serde(flatten)]
    pub project: ApiProject,
    pub tags: Vec<ApiTag>,
    pub media: Vec<ApiProjectMedia>,
    pub status: ProjectStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub github_repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub demo_url: Option<String>,
    pub created_at: String,
    pub last_activity: String,
    /// Authored primary label; the rail's "Type" slot (replaces "Language").
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub project_type: Option<String>,
    /// When true the page hides repo links and shows the closed-source callout.
    pub source_closed: bool,
    /// Explicit accent hex; frontend falls back to `#71717a` when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub accent_color: Option<String>,
    /// Last successful GitHub poll (RFC 3339); absent until first sync. Admin-only
    /// sync-health signal, distinct from `lastActivity`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub github_synced_at: Option<String>,
    /// Most recent GitHub sync failure; absent when healthy. Lets the admin UI
    /// flag a repo whose sync is broken.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub github_sync_error: Option<String>,
}

/// Single-project response that additionally carries the rich detail content.
///
/// Returned only by the single-project endpoint (`GET /api/projects/{ref}`) —
/// never in list responses, so the homepage payload stays free of every
/// project's full `ProseMirror` JSON. Feeds both the admin editor and the public
/// `/projects/{slug}` page.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ApiProjectDetail {
    #[serde(flatten)]
    pub project: ApiAdminProject,
    /// Last authored edit (RFC 3339), for a "last edited" signal in the editor.
    pub updated_at: String,
    /// `ProseMirror`/`TipTap` document JSON, or null when the project has no detail page.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "unknown")]
    pub detail_content: Option<serde_json::Value>,
    /// Authored CLI-hero transcript; absent for non-CLI projects.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub terminal_cast: Option<TerminalCast>,
    /// Curated related projects in authored order (resolved server-side from the
    /// `project_relations` table). Empty when none are authored.
    pub related: Vec<super::relations::ApiRelatedProject>,
}

impl DbProject {
    pub fn to_api_project(&self) -> ApiProject {
        let mut links = Vec::new();

        if let Some(ref repo) = self.github_repo {
            links.push(ApiProjectLink {
                url: format!("https://github.com/{repo}"),
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
    ) -> AppResult<ApiAdminProject> {
        let last_activity = self
            .last_github_activity
            .unwrap_or(self.created_at)
            .format(&Rfc3339)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let github_synced_at = self
            .github_synced_at
            .map(|t| t.format(&Rfc3339))
            .transpose()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(ApiAdminProject {
            project: self.to_api_project(),
            tags: tags.into_iter().map(|t| t.to_api_tag()).collect(),
            media: media.into_iter().map(|m| m.to_api_media()).collect(),
            status: self.status,
            github_repo: self.github_repo.clone(),
            demo_url: self.demo_url.clone(),
            created_at: self
                .created_at
                .format(&Rfc3339)
                .map_err(|e| AppError::Internal(e.to_string()))?,
            last_activity,
            project_type: self.project_type.clone(),
            source_closed: self.source_closed,
            accent_color: self.accent_color.clone(),
            github_synced_at,
            github_sync_error: self.github_sync_error.clone(),
        })
    }

    pub fn to_api_project_detail(
        &self,
        tags: Vec<DbTag>,
        media: Vec<DbProjectMedia>,
        related: Vec<super::relations::ApiRelatedProject>,
    ) -> AppResult<ApiProjectDetail> {
        Ok(ApiProjectDetail {
            project: self.to_api_admin_project(tags, media)?,
            updated_at: self
                .updated_at
                .format(&Rfc3339)
                .map_err(|e| AppError::Internal(e.to_string()))?,
            detail_content: self.detail_content.clone(),
            terminal_cast: self
                .terminal_cast
                .as_ref()
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
            related,
        })
    }
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AdminStats {
    pub total_projects: i32,
    #[ts(type = "Record<string, number>")]
    pub projects_by_status: serde_json::Value,
    pub total_tags: i32,
}

pub async fn get_public_projects(pool: &PgPool) -> Result<Vec<DbProject>, sqlx::Error> {
    query_as!(
        DbProject,
        r#"
        SELECT
            id,
            slug,
            name,
            short_description,
            status as "status: ProjectStatus",
            github_repo,
            demo_url,
            last_github_activity,
            created_at,
            updated_at,
            detail_content,
            project_type,
            source_closed,
            terminal_cast,
            accent_color,
            github_synced_at,
            github_sync_error
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
            status as "status: ProjectStatus",
            github_repo,
            demo_url,
            last_github_activity,
            created_at,
            updated_at,
            detail_content,
            project_type,
            source_closed,
            terminal_cast,
            accent_color,
            github_synced_at,
            github_sync_error
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
            status as "status: ProjectStatus",
            github_repo,
            demo_url,
            last_github_activity,
            created_at,
            updated_at,
            detail_content,
            project_type,
            source_closed,
            terminal_cast,
            accent_color,
            github_synced_at,
            github_sync_error
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
            status as "status: ProjectStatus",
            github_repo,
            demo_url,
            last_github_activity,
            created_at,
            updated_at,
            detail_content,
            project_type,
            source_closed,
            terminal_cast,
            accent_color,
            github_synced_at,
            github_sync_error
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

/// Field values for creating or updating a project (tags and relations are
/// handled separately). Groups the column set so the create/update query
/// functions don't carry a dozen positional arguments.
pub struct ProjectInput<'a> {
    pub name: &'a str,
    pub slug_override: Option<&'a str>,
    pub short_description: &'a str,
    pub status: ProjectStatus,
    pub github_repo: Option<&'a str>,
    pub demo_url: Option<&'a str>,
    pub detail_content: Option<&'a serde_json::Value>,
    pub project_type: Option<&'a str>,
    pub source_closed: bool,
    pub terminal_cast: Option<&'a serde_json::Value>,
    pub accent_color: Option<&'a str>,
}

/// Create project (without tags/relations - those are handled separately)
pub async fn create_project(
    pool: &PgPool,
    input: ProjectInput<'_>,
) -> Result<DbProject, sqlx::Error> {
    let slug = input
        .slug_override
        .map_or_else(|| slugify(input.name), slugify);

    query_as!(
        DbProject,
        r#"
        INSERT INTO projects (slug, name, short_description, status, github_repo, demo_url, detail_content, project_type, source_closed, terminal_cast, accent_color)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, slug, name, short_description, status as "status: ProjectStatus",
                  github_repo, demo_url, last_github_activity, created_at, updated_at, detail_content,
                  project_type, source_closed, terminal_cast, accent_color,
                  github_synced_at, github_sync_error
        "#,
        slug,
        input.name,
        input.short_description,
        input.status as ProjectStatus,
        input.github_repo,
        input.demo_url,
        input.detail_content as Option<&serde_json::Value>,
        input.project_type,
        input.source_closed,
        input.terminal_cast as Option<&serde_json::Value>,
        input.accent_color
    )
    .fetch_one(pool)
    .await
}

/// Update project (without tags/relations - those are handled separately)
pub async fn update_project(
    pool: &PgPool,
    id: Uuid,
    input: ProjectInput<'_>,
) -> Result<DbProject, sqlx::Error> {
    let slug = input
        .slug_override
        .map_or_else(|| slugify(input.name), slugify);

    query_as!(
        DbProject,
        r#"
        UPDATE projects
        SET slug = $2, name = $3, short_description = $4,
            status = $5, github_repo = $6, demo_url = $7, detail_content = $8,
            project_type = $9, source_closed = $10, terminal_cast = $11, accent_color = $12
        WHERE id = $1
        RETURNING id, slug, name, short_description, status as "status: ProjectStatus",
                  github_repo, demo_url, last_github_activity, created_at, updated_at, detail_content,
                  project_type, source_closed, terminal_cast, accent_color,
                  github_synced_at, github_sync_error
        "#,
        id,
        slug,
        input.name,
        input.short_description,
        input.status as ProjectStatus,
        input.github_repo,
        input.demo_url,
        input.detail_content as Option<&serde_json::Value>,
        input.project_type,
        input.source_closed,
        input.terminal_cast as Option<&serde_json::Value>,
        input.accent_color
    )
    .fetch_one(pool)
    .await
}

/// Persist a project's detail-content document, replacing only the
/// `detail_content` column. `content` is `None` for an empty document (see
/// [`crate::pm::Doc::to_stored`]), which stores SQL `NULL` so the project reads
/// as having no detail page.
pub async fn update_project_content(
    pool: &PgPool,
    id: Uuid,
    content: Option<&serde_json::Value>,
) -> Result<(), sqlx::Error> {
    query!(
        "UPDATE projects SET detail_content = $1 WHERE id = $2",
        content as Option<&serde_json::Value>,
        id
    )
    .execute(pool)
    .await?;
    Ok(())
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

/// Get all projects that have a `github_repo` set (for GitHub sync).
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
            status as "status: ProjectStatus",
            github_repo,
            demo_url,
            last_github_activity,
            created_at,
            updated_at,
            detail_content,
            project_type,
            source_closed,
            terminal_cast,
            accent_color,
            github_synced_at,
            github_sync_error
        FROM projects
        WHERE github_repo IS NOT NULL
        ORDER BY last_github_activity DESC NULLS LAST
        "#
    )
    .fetch_all(pool)
    .await
}

/// Record a successful GitHub poll: stamp `github_synced_at`, clear any prior
/// `github_sync_error`, and advance `last_github_activity` to the newer of the
/// stored value and `activity`. `GREATEST` ignores NULLs, so a `None` activity
/// leaves the stored timestamp untouched and the value never regresses.
pub async fn record_github_sync_success(
    pool: &PgPool,
    id: Uuid,
    activity: Option<OffsetDateTime>,
) -> Result<(), sqlx::Error> {
    query!(
        r#"
        UPDATE projects
        SET last_github_activity = GREATEST(last_github_activity, $2::timestamptz),
            github_synced_at = now(),
            github_sync_error = NULL
        WHERE id = $1
        "#,
        id,
        activity
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Record a failed GitHub poll, storing the error message for the admin UI.
/// Leaves `github_synced_at` (the last *successful* poll) untouched.
pub async fn record_github_sync_error(
    pool: &PgPool,
    id: Uuid,
    error: &str,
) -> Result<(), sqlx::Error> {
    query!(
        "UPDATE projects SET github_sync_error = $2 WHERE id = $1",
        id,
        error
    )
    .execute(pool)
    .await?;
    Ok(())
}
