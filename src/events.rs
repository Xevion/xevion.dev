use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;
use ts_rs::TS;
use uuid::Uuid;

/// Event severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize, TS)]
#[sqlx(type_name = "event_level", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum EventLevel {
    Info,
    Warning,
    Error,
}

/// All event types in the system, serialized as dot-separated strings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum EventType {
    #[serde(rename = "project.created")]
    ProjectCreated,
    #[serde(rename = "project.updated")]
    ProjectUpdated,
    #[serde(rename = "project.deleted")]
    ProjectDeleted,
    #[serde(rename = "project.tag_added")]
    ProjectTagAdded,
    #[serde(rename = "project.tag_removed")]
    ProjectTagRemoved,
    #[serde(rename = "tag.created")]
    TagCreated,
    #[serde(rename = "tag.updated")]
    TagUpdated,
    #[serde(rename = "tag.deleted")]
    TagDeleted,
    #[serde(rename = "settings.updated")]
    SettingsUpdated,
    #[serde(rename = "github.sync_completed")]
    GithubSyncCompleted,
    #[serde(rename = "github.sync_failed")]
    GithubSyncFailed,
    #[serde(rename = "github.rate_limited")]
    GithubRateLimited,
    #[serde(rename = "og.generated")]
    OgImageGenerated,
    #[serde(rename = "og.failed")]
    OgImageFailed,
    #[serde(rename = "cache.invalidated")]
    CacheInvalidated,
}

impl EventType {
    /// Get the string representation for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProjectCreated => "project.created",
            Self::ProjectUpdated => "project.updated",
            Self::ProjectDeleted => "project.deleted",
            Self::ProjectTagAdded => "project.tag_added",
            Self::ProjectTagRemoved => "project.tag_removed",
            Self::TagCreated => "tag.created",
            Self::TagUpdated => "tag.updated",
            Self::TagDeleted => "tag.deleted",
            Self::SettingsUpdated => "settings.updated",
            Self::GithubSyncCompleted => "github.sync_completed",
            Self::GithubSyncFailed => "github.sync_failed",
            Self::GithubRateLimited => "github.rate_limited",
            Self::OgImageGenerated => "og.generated",
            Self::OgImageFailed => "og.failed",
            Self::CacheInvalidated => "cache.invalidated",
        }
    }
}

/// Event to be written to the database (sent through channel)
#[derive(Debug)]
pub struct NewEvent {
    pub event_type: EventType,
    pub level: EventLevel,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub actor: Option<String>,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
}

/// API response type for events
#[derive(Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ApiEvent {
    pub id: String,
    pub event_type: String,
    pub level: EventLevel,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub actor: Option<String>,
    pub message: String,
    #[ts(type = "Record<string, unknown> | null")]
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
}

pub type EventSender = mpsc::Sender<NewEvent>;

const FLUSH_INTERVAL: Duration = Duration::from_millis(500);
const FLUSH_BATCH_SIZE: usize = 50;
const CHANNEL_BUFFER: usize = 1024;

/// Create a new event channel pair
pub fn create_channel() -> (EventSender, mpsc::Receiver<NewEvent>) {
    mpsc::channel(CHANNEL_BUFFER)
}

/// Fire-and-forget event logging. Drops the event if the channel is full.
#[allow(clippy::too_many_arguments)]
pub fn log_event(
    sender: &EventSender,
    event_type: EventType,
    level: EventLevel,
    entity_type: Option<&str>,
    entity_id: Option<Uuid>,
    actor: Option<&str>,
    message: String,
    metadata: Option<serde_json::Value>,
) {
    let event = NewEvent {
        event_type,
        level,
        entity_type: entity_type.map(String::from),
        entity_id,
        actor: actor.map(String::from),
        message,
        metadata,
    };

    if let Err(e) = sender.try_send(event) {
        tracing::warn!(error = %e, "Failed to enqueue event (channel full or closed)");
    }
}

/// Background task that reads events from the channel and batch-inserts them
pub async fn run_event_writer(mut receiver: mpsc::Receiver<NewEvent>, pool: sqlx::PgPool) {
    let mut buffer: Vec<NewEvent> = Vec::with_capacity(FLUSH_BATCH_SIZE);
    let mut flush_timer = tokio::time::interval(FLUSH_INTERVAL);
    flush_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            event = receiver.recv() => {
                match event {
                    Some(e) => {
                        buffer.push(e);
                        if buffer.len() >= FLUSH_BATCH_SIZE {
                            flush_events(&pool, &mut buffer).await;
                        }
                    }
                    None => {
                        // Channel closed, flush remaining and exit
                        if !buffer.is_empty() {
                            flush_events(&pool, &mut buffer).await;
                        }
                        tracing::debug!("Event writer shutting down");
                        return;
                    }
                }
            }
            _ = flush_timer.tick() => {
                if !buffer.is_empty() {
                    flush_events(&pool, &mut buffer).await;
                }
            }
        }
    }
}

async fn flush_events(pool: &sqlx::PgPool, buffer: &mut Vec<NewEvent>) {
    let events: Vec<NewEvent> = std::mem::take(buffer);
    if let Err(e) = crate::db::events::batch_insert_events(pool, &events).await {
        tracing::error!(error = %e, count = events.len(), "Failed to flush events to database");
    }
}
