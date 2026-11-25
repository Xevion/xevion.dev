# TypeScript Style Guide

Conventions for the Worker (`src/lib/server/**`, `src/routes/**/+server.ts`) and the CLI (`cli/**`). Svelte-specific rules live in `SVELTE.md`.

## Formatting and Imports

Formatting is automated. Do not argue about it.

- Prettier with `prettier-plugin-svelte` (`just format`), then ESLint with `typescript-eslint` and `eslint-plugin-svelte`. `just check` runs both along with `svelte-check`.
- Generated files (`styled-system/`, `worker-configuration.d.ts`) are excluded from the formatter and linter. Never hand-edit them.
- Import project code through the `#lib/...` subpath imports with an explicit `.ts` extension: `import { ok } from "#lib/server/api.ts"`. Inside `cli/`, sibling files use relative imports (`./errors.ts`).
- Use `import type` for type-only imports.
- Bindings come from `import { env, waitUntil } from "cloudflare:workers"`, never from `event.platform`.
- Modules that are unit tested must not import `$app/*`; vitest runs without the SvelteKit plugin.

## Domain Vocabulary

Use these canonical names consistently.

| Concept                  | TypeScript (wire)   | Database (`schema.ts`) |
| ------------------------ | ------------------- | ---------------------- |
| A portfolio entry        | `ApiProject`        | `projects`             |
| A categorization label   | `ApiTag`            | `tags`                 |
| Project-tag relationship | (inline `tags`)     | `project_tags`         |
| Project-project relation | `ApiRelatedProject` | `project_relations`    |
| Tag relatedness          | `ApiRelatedTag`     | (computed)             |
| A media attachment       | `ApiProjectMedia`   | `project_media`        |
| Site identity settings   | `ApiSiteIdentity`   | `site_identity`        |
| An external link         | `ApiSocialLink`     | `social_links`         |
| A login session          | `ApiSession`        | `sessions`             |
| Project activity stage   | `ProjectStatus`     | `projects.status`      |

Wire types carry an `Api` prefix. Drizzle tables are plural camelCase exports (`projectTags`) over snake_case SQL names. When in doubt, use the singular form of the domain noun.

## Comments

- Explain **why**, not **what**. The code already says what.
- Never reference old implementations, migrations, or refactoring history.
- Never add banner comments (`// ========`, `// ------`).
- Doc comments open with one self-contained summary line.
- TODOs must include context: `// TODO(auth): rate-limit login attempts`

```typescript
// Bad
// Convert project to API format
export function toApiProject(p: ProjectRow): ApiProject { ... }

// Good
// Public responses omit description and status to keep list payloads small
export function toApiProject(p: ProjectRow): ApiProject { ... }
```

## Logging

Server code uses LogTape through `getAppLogger(...category)` from `#lib/logger.ts`, which prefixes the `xevion` category. Browser code uses `getLogger(["admin", ...])` from `@logtape/logtape`. The CLI does not log; see CLI Output below.

```typescript
const logger = getAppLogger("projects");
```

### Static Messages, Structured Fields

Messages are static strings. Put all dynamic data in the properties object, with camelCase keys.

```typescript
// Good
logger.warn("rendered content not stored", {
  projectId: row.id,
  cause: String(cause),
});

// Bad
logger.warn(`Could not store rendered content for ${row.id}`);
```

### Log Levels

| Level   | Meaning                                  | Example                                        |
| ------- | ---------------------------------------- | ---------------------------------------------- |
| `error` | Requires attention, something is broken  | Unhandled request error, manual sync failure   |
| `warn`  | Recoverable, but unexpected              | Failed login, rate limit hit, stale content    |
| `info`  | Lifecycle events, operational milestones | Login, GitHub sync run, daily cleanup          |
| `debug` | Routine operations                       | Shown in dev only (production floor is `info`) |

### Standard Field Names

| Field       | Meaning                                     |
| ----------- | ------------------------------------------- |
| `cause`     | Caught value, stringified (`String(cause)`) |
| `error`     | An `ApiError` or error message              |
| `projectId` | Project UUID                                |
| `username`  | Account involved in an auth event           |
| `method`    | HTTP method                                 |
| `status`    | HTTP status code                            |
| `route`     | SvelteKit route id or URL path              |

Never log secrets, tokens, cookies or credential-bearing URLs.

## Error Philosophy

- **Expected errors** (not found, validation failure, auth required) are raised as typed API errors with a stable code.
- **Unexpected errors** (D1 failure, upstream outage) are logged with context and surfaced as a generic `INTERNAL` response. `handleError` in `src/hooks.server.ts` assigns an `errorId` so the log line can be matched to the client report.
- **Never swallow errors.** Log and propagate, or log and return a meaningful response. Background work under `waitUntil` attaches a `.catch` that logs.
- **User-facing messages are separate from internal ones.** Never leak stack traces, SQL errors, or internal state to the client.

### Where Each Layer Throws or Returns

| Layer                                | Mechanism                                                                                           |
| ------------------------------------ | --------------------------------------------------------------------------------------------------- |
| `src/lib/server/**` and `+server.ts` | Throw `apiError(status, code, message, fieldErrors?)` or `notFound(what)` from `#lib/server/api.ts` |
| Page and admin loads                 | `error(status, message)` from `@sveltejs/kit`                                                       |
| Browser API calls (`src/lib/api.ts`) | Return neverthrow `ResultAsync<T, ApiError>`; never throw                                           |
| CLI (`cli/**`)                       | Throw `CliError` (`cli/errors.ts`); `main.ts` renders it and sets the exit code                     |

### API Error Shape

```json
{
  "message": "Human-readable error message",
  "code": "MACHINE_READABLE_CODE",
  "fieldErrors": { "slug": "already taken" }
}
```

`fieldErrors` is optional. Codes are `UPPER_SNAKE_CASE`, stable, and listed in the `ApiErrorCode` union in `src/lib/server/api.ts`. Clients may match on them. Status codes follow standard semantics: `400` validation, `401` unauthenticated, `403` forbidden, `404` not found, `409` conflict, `412` failed `If-Match`, `429` rate limited, `500` internal.

Under `/api/*`, `jsonifyErrorResponse` converts any non-JSON error response (including bare framework 404/405s) into this shape.

## API Design

### Handlers

Handlers in `+server.ts` stay thin: parse, call a `src/lib/server/*` function, wrap the result.

```typescript
export const POST: RequestHandler = async ({ request, locals }) => {
  const input = await parseJsonBody(request, ProjectInputSchema);
  return ok(await createProject(locals.db, input), { status: 201 });
};
```

- Request bodies are validated with arktype schemas through `parseJsonBody`, which fails the request with a 400 `VALIDATION_ERROR`.
- Use `Response.json` (never the deprecated `json()`), and `ok(data, init?)` for success.
- Route access is decided in `src/lib/server/access.ts`, not in handlers. Handlers read `locals.user` only to shape admin-aware output.
- D1 has no interactive transactions: use `db.batch([...])` and respect the 100 bound-parameter limit (`src/lib/server/chunk.ts`).

### JSON Conventions

- Field names: `camelCase`.
- Success envelope: every endpoint returns `{ "data": ... }`, for lists, single resources and mutations alike. Mutations return the created or updated resource; creation uses `201`.
- Dates: ISO 8601 strings from `toISOString()`. Database timestamps are integer seconds (Drizzle `mode: 'timestamp'`), converted at the mapping layer (`src/lib/server/project-mapping.ts`).
- IDs: string UUIDs.
- Nullability: optional fields are omitted when empty, not sent as `null`. Database rows use `null`; mapping functions drop them.

### URL Design

- Resource IDs in the path: `/api/projects/{ref}`.
- HTTP verbs: `GET` read, `POST` create, `PUT` update, `DELETE` delete.
- `{ref}` parameters accept both a UUID and a slug.

## Wire Types

Types shared between the Worker, browser and CLI live in `src/lib/bindings/` and are maintained by hand, one type per file, re-exported from `index.ts`.

- Import with `import type { ApiProject } from "#lib/bindings/index.ts"`.
- Update the binding in the same change as the mapper that produces it.
- Request types that exist only for the admin client (`CreateProjectData`, ...) live in `src/lib/admin-types.ts`. Server-side request schemas are arktype schemas next to the code that consumes them (`ProjectInputSchema` in `src/lib/server/project-mutations.ts`).
- The CLI keeps its own minimal copies in `cli/types.ts` of only the fields it reads, so the compiled binary does not pull in server code. It does share `src/lib/tiptap/*` with the server.

## CLI Conventions

- Commands are `CommandNode` objects (`cli/framework.ts`) grouped per resource (`cli/projects.ts`, `cli/tags.ts`, `cli/settings.ts`) and assembled in `cli/main.ts`. Flags are declared with cleye.
- Global flags are `--api`, `--config` and `--json`.
- Stdout carries only data. Status, progress and confirmations go to stderr through the helpers in `cli/output.ts`. Under `--json`, errors are also printed as JSON on stdout.
- Every failure is a `CliError` built with a static constructor (`CliError.invalid`, `CliError.connect`, ...), carrying a `code`, optional `help` text and an exit code.
- `ApiClient` (`cli/client.ts`) authenticates with a bearer token, unwraps the `{ data }` envelope, and turns non-2xx responses into `CliError`s.

## Testing

### Philosophy

- Test **behavior**, not implementation. Assert what the code does, not how.
- Prefer real dependencies over mocks when practical: database tests run against a real (local) D1.
- Test names read as specifications, and a test earns its place by exercising real logic or a bug that happened.

```typescript
test("rejects a slug that collides with an existing project", async () => { ... });
```

### Setup

`just test` runs Vitest with two projects (`vitest.config.ts`):

- `unit`: Node environment, `src/**/*.test.ts` and `cli/**/*.test.ts`, excluding `*.db.test.ts`. Pure logic such as parsers, mappers, negotiation and argument handling.
- `db`: workerd with a migrated local D1 (`@cloudflare/vitest-plugin`), `src/**/*.db.test.ts`. Use `testDb()` and `resetDb()` from `src/lib/server/db/test-db.ts`; nothing rolls back between tests, so reset in `beforeEach`. These files run serially.

Tests sit beside the code they cover. There is no component test setup; keep logic worth testing in plain `.ts` modules.
