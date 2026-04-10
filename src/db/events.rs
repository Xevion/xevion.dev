use sqlx::PgPool;
use uuid::Uuid;

use crate::events::{ApiEvent, EventLevel, NewEvent};

/// Batch-insert events into the database using unnest for efficient multi-row insert
pub async fn batch_insert_events(pool: &PgPool, events: &[NewEvent]) -> Result<(), sqlx::Error> {
    if events.is_empty() {
        return Ok(());
    }

    let mut event_types: Vec<String> = Vec::with_capacity(events.len());
    let mut levels: Vec<EventLevel> = Vec::with_capacity(events.len());
    let mut entity_types: Vec<Option<String>> = Vec::with_capacity(events.len());
    let mut entity_ids: Vec<Option<Uuid>> = Vec::with_capacity(events.len());
    let mut actors: Vec<Option<String>> = Vec::with_capacity(events.len());
    let mut messages: Vec<String> = Vec::with_capacity(events.len());
    let mut metadatas: Vec<Option<serde_json::Value>> = Vec::with_capacity(events.len());

    for event in events {
        event_types.push(event.event_type.as_str().to_string());
        levels.push(event.level);
        entity_types.push(event.entity_type.clone());
        entity_ids.push(event.entity_id);
        actors.push(event.actor.clone());
        messages.push(event.message.clone());
        metadatas.push(event.metadata.clone());
    }

    sqlx::query!(
        r#"
        INSERT INTO events (event_type, level, entity_type, entity_id, actor, message, metadata)
        SELECT * FROM UNNEST(
            $1::text[],
            $2::event_level[],
            $3::text[],
            $4::uuid[],
            $5::text[],
            $6::text[],
            $7::jsonb[]
        )
        "#,
        &event_types,
        &levels as &[EventLevel],
        &entity_types as &[Option<String>],
        &entity_ids as &[Option<Uuid>],
        &actors as &[Option<String>],
        &messages,
        &metadatas as &[Option<serde_json::Value>],
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Query parameters for filtering events
#[derive(Debug, Default)]
pub struct EventFilters {
    pub limit: i64,
    pub offset: i64,
    pub level: Option<EventLevel>,
    pub entity_type: Option<String>,
    pub event_type: Option<String>,
}

/// Fetch events with optional filters, ordered by `created_at` DESC
pub async fn get_events(
    pool: &PgPool,
    filters: EventFilters,
) -> Result<Vec<ApiEvent>, sqlx::Error> {
    let rows = sqlx::query_as!(
        DbEvent,
        r#"
        SELECT
            id,
            event_type,
            level as "level: EventLevel",
            entity_type,
            entity_id,
            actor,
            message,
            metadata,
            created_at
        FROM events
        WHERE ($1::event_level IS NULL OR level = $1)
          AND ($2::text IS NULL OR entity_type = $2)
          AND ($3::text IS NULL OR event_type = $3)
        ORDER BY created_at DESC
        LIMIT $4 OFFSET $5
        "#,
        filters.level as Option<EventLevel>,
        filters.entity_type,
        filters.event_type,
        filters.limit,
        filters.offset,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(DbEvent::into_api).collect())
}

struct DbEvent {
    id: Uuid,
    event_type: String,
    level: EventLevel,
    entity_type: Option<String>,
    entity_id: Option<Uuid>,
    actor: Option<String>,
    message: String,
    metadata: Option<serde_json::Value>,
    created_at: time::OffsetDateTime,
}

impl DbEvent {
    fn into_api(self) -> ApiEvent {
        ApiEvent {
            id: self.id.to_string(),
            event_type: self.event_type,
            level: self.level,
            entity_type: self.entity_type,
            entity_id: self.entity_id.map(|id| id.to_string()),
            actor: self.actor,
            message: self.message,
            metadata: self.metadata,
            created_at: self
                .created_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
        }
    }
}
