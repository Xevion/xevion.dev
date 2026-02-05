# Cross-Stack Style Guide

Shared conventions that apply across both Rust and SvelteKit.

## Formatting

Formatting is automated. Do not argue about it.

- **Rust:** `rustfmt` (default config)
- **TypeScript/Svelte:** Prettier with `prettier-plugin-svelte`
- **SQL:** Follow sqlx inline query conventions (no separate formatter)

Run `just check` to verify all formatting and linting in one pass.

## Domain Vocabulary

Use these canonical names consistently across both stacks.

| Concept | Rust | TypeScript | Database |
|---|---|---|---|
| A portfolio entry | `Project` | `Project` | `projects` |
| A categorization label | `Tag` | `Tag` | `tags` |
| Project-tag relationship | `ProjectTag` | (inline) | `project_tags` |
| Tag relatedness metric | `TagCooccurrence` | `RelatedTag` | `tag_cooccurrence` |
| A media attachment | `ProjectMedia` | `ProjectMedia` | `project_media` |
| Site identity settings | `SiteIdentity` | `SiteIdentity` | `site_identity` |
| An external link | `SocialLink` | `SocialLink` | `social_links` |
| Current user session | `Session` | (cookie-based) | `sessions` |
| Project lifecycle stage | `ProjectStatus` | `ProjectStatus` | `project_status` enum |

When in doubt, use the singular form of the domain noun.

## Comments

- Explain **why**, not **what**. The code already says what.
- Never reference old implementations, migrations, or refactoring history.
- Never add banner comments (`// ========`, `// ------`).
- TODOs are acceptable but must include context: `// TODO(auth): rate-limit login attempts`

```rust
// Bad
// Convert project to API format
fn to_api_project(&self) -> ApiProject { ... }

// Good
// Public API omits description and status to keep list responses lightweight
fn to_api_project(&self) -> ApiProject { ... }
```

## Logging

### Static Messages, Structured Fields

Log messages must be static strings. Put all dynamic data in structured fields.

```rust
// Good
tracing::info!(project_id = %id, tag_count = tags.len(), "Project tags updated");

// Bad
tracing::info!("Updated {} tags for project {}", tags.len(), id);
```

```typescript
// Good
logger.info("Projects fetched", { count: projects.length });

// Bad
logger.info(`Fetched ${projects.length} projects`);
```

### Log Levels

| Level | Meaning | Example |
|---|---|---|
| `ERROR` | Requires attention, something is broken | DB connection lost, R2 upload failed |
| `WARN` | Recoverable, but unexpected | Session expired mid-request, cache miss on expected key |
| `INFO` | Lifecycle events, operational milestones | Server started, session created, OG image generated |
| `DEBUG` | Routine operations, request details | API request/response, cache hit/miss |
| `TRACE` | Verbose internals (rarely used) | Full request headers, SQL query text |

### Standard Field Names

Use these consistently across both stacks:

| Field | Type | Meaning |
|---|---|---|
| `duration_ms` | number | Operation timing |
| `count` | number | Item count |
| `bytes` | number | Payload size |
| `error` | string | Error message or display string |
| `project_id` | string | Project UUID |
| `tag_id` | string | Tag UUID |
| `session_id` | string | Session ULID |
| `request_id` | string | Request ULID |
| `status` | number | HTTP status code |
| `path` | string | URL path |
| `method` | string | HTTP method |

## Error Philosophy

### Errors Are Values

Both stacks treat errors as typed values, not exceptions to be caught ad-hoc.

- **Expected errors** (not found, validation failure, auth required) get typed representations.
- **Unexpected errors** (DB down, network failure) get wrapped with context.
- **Never swallow errors.** Log and propagate, or log and return a meaningful response.
- **User-facing error messages are separate from internal ones.** Never leak stack traces, SQL errors, or internal state to the client.

### API Error Shape

All API error responses follow this structure:

```json
{
  "error": "Human-readable error message",
  "code": "MACHINE_READABLE_CODE"
}
```

Error codes are `UPPER_SNAKE_CASE` and stable (clients can match on them).

HTTP status codes follow standard semantics:
- `400` — Validation failure, malformed request
- `401` — Not authenticated
- `403` — Authenticated but not authorized
- `404` — Resource not found
- `409` — Conflict (duplicate slug, etc.)
- `500` — Internal server error

## API Design

### JSON Conventions

- Field names: `camelCase` (enforced by `#[serde(rename_all = "camelCase")]` in Rust)
- Dates: ISO 8601 / RFC 3339 strings (`2024-01-15T09:30:00Z`)
- IDs: String UUIDs (not raw binary)
- Nullability: Omit null optional fields (`#[serde(skip_serializing_if = "Option::is_none")]`)

### Response Shapes

**List endpoints** return arrays directly:
```json
[{ "id": "...", "name": "..." }, ...]
```

**Single resource endpoints** return the object directly:
```json
{ "id": "...", "name": "..." }
```

**Mutation endpoints** return the created/updated resource.

### URL Design

- Resource IDs in URL path: `/api/projects/{ref}`
- Use HTTP verbs: `GET` (read), `POST` (create), `PUT` (update), `DELETE` (delete)
- Universal identifiers: `{ref}` parameters accept both UUID and slug

## Type Sharing (ts-rs)

Rust API types are the source of truth. TypeScript types are generated from Rust structs using `ts-rs`.

- Only API-facing types get `#[derive(TS)] #[ts(export)]` — database models stay internal.
- Generated bindings live in `web/src/lib/bindings/`.
- Regenerate with `just bindings` (or `cargo test export_bindings`).
- Never hand-edit generated files.
- Import generated types as `import type { Project } from '$lib/bindings'`.

## Testing

### Philosophy

- Test **behavior**, not implementation. Assert what the code does, not how.
- Prefer integration tests over mocking when practical.
- Use descriptive test names that read as specifications:

```rust
#[test]
fn rejects_project_with_empty_name() { ... }
```

```typescript
test("returns null for non-existent project", async () => { ... });
```

### Stack-Specific

- **Rust:** `cargo nextest run`. Integration tests in `tests/`, unit tests alongside code.
- **TypeScript:** Vitest. API client and utility tests in `*.test.ts` files alongside source.
- **Components:** Vitest with `@testing-library/svelte` for component behavior tests.
