use serde::{Deserialize, Serialize};
use sqlx::{PgPool, query};
use ts_rs::TS;
use uuid::Uuid;

/// Compact view of a related project, resolved for the detail page's "Related
/// work" list. Carries just what the row renders: name, slug, the authored
/// `projectType`, and the accent (for the generated cover thumbnail).
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ApiRelatedProject {
    pub id: String,
    pub slug: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub project_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub accent_color: Option<String>,
}

/// Curated related projects for a project, in authored order.
///
/// Hidden projects are excluded so a relation can't leak an unlisted project
/// onto a public page.
pub async fn get_related_projects(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<Vec<ApiRelatedProject>, sqlx::Error> {
    let rows = query!(
        r#"
        SELECT p.id, p.slug, p.name, p.project_type, p.accent_color
        FROM project_relations r
        JOIN projects p ON p.id = r.related_project_id
        WHERE r.project_id = $1 AND p.hidden = false
        ORDER BY r.position ASC
        "#,
        project_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| ApiRelatedProject {
            id: row.id.to_string(),
            slug: row.slug,
            name: row.name,
            project_type: row.project_type,
            accent_color: row.accent_color,
        })
        .collect())
}

/// Replace a project's related list with `related_ids`, in the given order.
///
/// Full replace (delete + re-insert) rather than a diff, since authored order
/// is the point. Self-references are filtered out (the table also CHECKs it).
pub async fn set_project_relations(
    pool: &PgPool,
    project_id: Uuid,
    related_ids: &[Uuid],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    query!(
        "DELETE FROM project_relations WHERE project_id = $1",
        project_id
    )
    .execute(&mut *tx)
    .await?;

    for (position, related_id) in related_ids
        .iter()
        .filter(|id| **id != project_id)
        .enumerate()
    {
        query!(
            r#"
            INSERT INTO project_relations (project_id, related_project_id, position)
            VALUES ($1, $2, $3)
            ON CONFLICT (project_id, related_project_id) DO UPDATE SET position = EXCLUDED.position
            "#,
            project_id,
            related_id,
            position as i32
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
