# Rust Style Guide

Conventions for the Rust backend (`src/`).

## Module Organization

```
src/
├── main.rs          Entry point, CLI dispatch
├── state.rs         AppState, AppError
├── routes.rs        Route definitions and wiring
├── config.rs        CLI and environment config types
│
├── handlers/        HTTP request handlers (thin orchestration)
├── db/              Database types, queries, and conversions
├── middleware/       Axum middleware (request ID, etc.)
├── cli/             CLI subcommands and API client
│
├── auth.rs          Session management, password hashing
├── cache.rs         ISR cache (moka, stale-while-revalidate)
├── proxy.rs         Bun SSR proxy logic
├── http.rs          HTTP client (TCP/Unix socket)
├── assets.rs        Embedded asset serving
├── og.rs            OG image orchestration
├── r2.rs            Cloudflare R2 client
├── health.rs        Health check coordination
├── tarpit.rs        Malicious path detection
├── formatter.rs     Log formatters (JSON + pretty)
└── utils.rs         Shared helpers
```

### Layer Rules

```
handlers/  →  db/  →  PostgreSQL
    ↓
  auth.rs, cache.rs, og.rs  (side effects)
```

- **Handlers** orchestrate: extract request, check auth, call db, trigger side effects, return response.
- **`db/`** owns all SQL queries, database types, and type conversions. No HTTP concepts.
- Handlers may call `db/` functions directly for simple CRUD. There is no mandatory services layer.
- Background tasks (OG generation, session cleanup, GitHub sync) live in dedicated modules, not handlers.

## Error Handling

### AppError Enum

All handlers return `Result<T, AppError>`. One enum covers all error cases.

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found")]
    NotFound,

    #[error("Authentication required")]
    Unauthorized,

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}
```

### IntoResponse

`AppError` implements `IntoResponse` to map variants to HTTP status codes and the standard error shape:

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", self.to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", self.to_string()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone()),
            AppError::Database(err) => {
                tracing::error!(error = %err, "Database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal server error".into())
            }
            AppError::Internal(err) => {
                tracing::error!(error = %err, "Internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal server error".into())
            }
        };

        (status, Json(json!({ "error": message, "code": code }))).into_response()
    }
}
```

### From Conversions

Use `#[from]` for automatic conversion from library errors. For domain-specific mappings, implement `From` manually:

```rust
// Automatic: sqlx::Error → AppError::Database
// Manual: unique violation → AppError::Conflict
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
                AppError::Conflict("Resource already exists".into())
            }
            _ => AppError::Database(err),
        }
    }
}
```

### Result Alias

```rust
pub type AppResult<T> = Result<T, AppError>;
```

Handlers use this consistently:

```rust
pub async fn get_project_handler(
    State(state): State<Arc<AppState>>,
    Path(ref_): Path<String>,
) -> AppResult<Json<ApiAdminProject>> {
    let project = db::get_project_by_ref(&state.pool, &ref_)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(project.to_api_admin_project(tags, media)))
}
```

## Validation

### Typed Extractors

Validation happens before handler logic runs, using custom axum extractors.

```rust
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::Validation(e.to_string()))?;

        value.validate().map_err(|e| AppError::Validation(e.to_string()))?;

        Ok(ValidatedJson(value))
    }
}
```

Request types implement a `Validate` trait or use `validator` crate derives:

```rust
#[derive(Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub slug: Option<String>,
    pub short_description: String,
    // ...
}

impl Validate for CreateProjectRequest {
    fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Name cannot be empty".into());
        }
        Ok(())
    }
}
```

## State

### AppState Pattern

Global state is wrapped in `Arc<AppState>` and shared via axum's `State` extractor.

```rust
pub struct AppState {
    pub pool: PgPool,
    pub session_manager: SessionManager,
    pub isr_cache: IsrCache,
    pub r2: Option<R2Client>,
    pub http_client: HttpClient,
    // ...
}
```

- Required services are direct fields.
- Optional services use `Option<T>` (e.g., R2 when credentials aren't configured).
- Access with `state.pool`, `state.r2.as_ref()`, etc.

## Database

### Compile-Time Checked Queries

All queries use `sqlx::query!()` or `sqlx::query_as!()` for compile-time SQL verification.

```rust
let project = sqlx::query_as!(
    DbProject,
    r#"SELECT id, slug, name, short_description, description,
              status as "status: ProjectStatus",
              github_repo, demo_url, last_github_activity, created_at
       FROM projects WHERE id = $1"#,
    id
)
.fetch_optional(pool)
.await?;
```

### Type Tiers

```
DbProject          (internal, sqlx::FromRow)
    ↓ to_api_project()
ApiProject          (public, Serialize + TS)
    ↓ to_api_admin_project(tags, media)
ApiAdminProject     (authenticated, Serialize + TS)
```

- `Db*` types match the database schema exactly. Internal only, never serialized to JSON.
- `Api*` types are the public contract. They get `#[derive(TS)] #[ts(export)]` for TypeScript binding generation.
- Conversion methods live on the `Db*` types.

### ts-rs Integration

API-facing types derive `TS` for automatic TypeScript binding generation:

```rust
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ApiProject {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub short_description: String,
    pub links: Vec<ApiProjectLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ApiAdminProject {
    #[serde(flatten)]
    pub project: ApiProject,
    pub tags: Vec<ApiTag>,
    pub media: Vec<ApiProjectMedia>,
    pub status: String,
    pub description: String,
    pub created_at: String,
    pub last_activity: String,
}
```

Rules:
- `DateTime` fields use `#[ts(type = "string")]` (serialized as ISO 8601 strings).
- `Db*` types never get `#[ts(export)]`.
- Regenerate bindings after changing any `Api*` struct.

### Serialization Conventions

- `#[serde(rename_all = "camelCase")]` on all API types.
- `#[serde(skip_serializing_if = "Option::is_none")]` on optional fields.
- `#[serde(flatten)]` when composing API types (e.g., `ApiAdminProject` embeds `ApiProject`).
- Dates are formatted as RFC 3339 strings, not raw `OffsetDateTime`.

## Axum Specifics

### Route Syntax

Axum v0.8+ uses curly braces for path parameters:

```rust
// Correct
.route("/api/projects/{id}", get(handler))
.route("/api/tags/{slug}/related", get(handler))

// Wrong (will not compile or route correctly)
.route("/api/projects/:id", get(handler))   // Express-style
.route("/api/projects/*id", get(handler))   // Old axum glob
```

### Universal Identifiers

Route parameters named `{ref}` accept both UUID and slug. The handler auto-detects:

```rust
fn parse_ref(ref_: &str) -> RefType {
    match Uuid::parse_str(ref_) {
        Ok(uuid) => RefType::Id(uuid),
        Err(_) => RefType::Slug(ref_.to_string()),
    }
}
```

## Async

- Runtime: `tokio` (multi-threaded).
- Background tasks use `tokio::spawn`. Never block the runtime.
- Never panic in async code. Return `Result` or log and continue.

## Logging

- Import `tracing` macros at the top of each file.
- Use `#[instrument]` on handlers for automatic span creation.
- Use structured fields, not format strings (see STYLE.md).

```rust
#[instrument(skip(state, jar))]
pub async fn create_project_handler(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    ValidatedJson(payload): ValidatedJson<CreateProjectRequest>,
) -> AppResult<(StatusCode, Json<ApiAdminProject>)> {
    tracing::info!(name = %payload.name, "Creating project");
    // ...
}
```

## ISR Cache

- Invalidate on mutation: project create/update/delete, tag changes, project-tag changes.
- Invalidation target is typically `"/"` (homepage).
- Never cache admin pages, API routes, internal routes, or static assets.

```rust
// After any project mutation
state.isr_cache.invalidate("/").await;
```

## Linting

- Zero clippy warnings. Enforced by `just check` with `-D warnings`.
- Prefer `clippy::pedantic` where practical, but don't `#[allow]` liberally to satisfy it.

## Testing

- Runner: `cargo nextest run`
- Integration tests: `tests/` directory
- Unit tests: `#[cfg(test)] mod tests` alongside the code they test
- Use `assert2` for assertions when available:

```rust
use assert2::assert;

#[test]
fn parses_uuid_ref() {
    let id = Uuid::new_v4();
    assert!(parse_ref(&id.to_string()) == RefType::Id(id));
}
```
