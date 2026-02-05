# SvelteKit Style Guide

Conventions for the SvelteKit frontend (`web/src/`).

## Route Organization

Follow SvelteKit's file-based routing conventions:

```
routes/
├── +page.svelte              Homepage
├── +page.server.ts           Homepage data loader
├── +layout.svelte            Root layout
├── +layout.server.ts         Root layout data (session, settings)
├── +error.svelte             Error page template
│
├── admin/
│   ├── +layout.server.ts     Auth guard
│   ├── +layout.svelte        Admin shell
│   ├── projects/             Project management
│   ├── tags/                 Tag management
│   └── settings/[[tab]]/     Settings with optional tab param
│
├── internal/
│   └── ogp/generate/         OG image generation (blocked externally)
│
└── errors/[code]/            Prerendered error pages
```

### Routing Rules

- One concern per route segment.
- `+page.server.ts` for all data loading (server loads only — see Data Loading below).
- `+layout.server.ts` for shared data (settings, session state).
- Group related routes under a directory (e.g., `admin/projects/`).

## Data Loading

### Server Loads Only

All data loading uses `+page.server.ts`, never `+page.ts` (universal loads).

The architecture requires this: SvelteKit talks to Rust over Unix sockets during SSR. The browser cannot reach that socket. Server loads ensure data always flows through the Bun process.

```typescript
// +page.server.ts
import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import type { Project } from "$lib/bindings";

export const load: PageServerLoad = async ({ fetch }) => {
  const result = await apiFetch<Project[]>("/api/projects", { fetch });

  // Unwrap Result in the load function — components receive clean data
  const projects = result.unwrapOrElse((err) => {
    throw error(err.status, err.message);
  });

  return { projects };
};
```

### Result Handling in Loads

`apiFetch` returns `Result<T, ApiError>` (true-myth). Unwrap in the load function so components receive plain typed data, not Results.

- **List pages:** Return an empty fallback on error (degrade gracefully).
- **Detail pages:** Throw `error()` to trigger SvelteKit's error page.
- **Admin pages:** Throw on any error (admin should see explicit failures).

```typescript
// List page: fallback to empty
export const load: PageServerLoad = async ({ fetch }) => {
  const result = await apiFetch<Project[]>("/api/projects", { fetch });
  return { projects: result.unwrapOr([]) };
};

// Detail page: throw on not found
export const load: PageServerLoad = async ({ fetch, params }) => {
  const result = await apiFetch<Project>(`/api/projects/${params.slug}`, { fetch });
  const project = result.unwrapOrElse((err) => {
    throw error(err.status, err.message);
  });
  return { project };
};
```

## API Client

### Architecture

Two API client layers:

| Module | Context | Transport |
|---|---|---|
| `$lib/api.server.ts` | SSR (load functions) | Unix socket or HTTP to Rust |
| `$lib/api.ts` | Browser (client-side calls) | HTTP to Rust via same origin |

### Result Pattern (true-myth)

The API client never throws. All methods return `Result<T, ApiError>`.

```typescript
import { Result, ok, err } from "true-myth/result";

async function fetchJson<T>(url: string, init?: RequestInit): Promise<Result<T, ApiError>> {
  try {
    const response = await fetch(url, init);
    if (!response.ok) {
      return err(ApiError.fromResponse(response));
    }
    return ok(await response.json());
  } catch (e) {
    return err(ApiError.network(e));
  }
}
```

### Endpoint Organization

Group API functions by resource. Each function returns `Promise<Result<T, ApiError>>`.

```typescript
// Projects API
export async function getProjects(): Promise<Result<Project[], ApiError>> { ... }
export async function getProject(ref: string): Promise<Result<AdminProject, ApiError>> { ... }
export async function createProject(data: CreateProjectData): Promise<Result<AdminProject, ApiError>> { ... }
export async function updateProject(data: UpdateProjectData): Promise<Result<AdminProject, ApiError>> { ... }
export async function deleteProject(id: string): Promise<Result<AdminProject, ApiError>> { ... }
```

### ApiError

```typescript
export class ApiError {
  constructor(
    public readonly status: number,
    public readonly message: string,
    public readonly code?: string,
  ) {}

  static fromResponse(res: Response): ApiError { ... }
  static network(cause: unknown): ApiError { ... }

  get isNotFound(): boolean { return this.status === 404; }
  get isAuthError(): boolean { return this.status === 401 || this.status === 403; }
  get isServerError(): boolean { return this.status >= 500; }
}
```

## Type System

### Generated Bindings

TypeScript types for API responses are generated from Rust via ts-rs. Import from `$lib/bindings`:

```typescript
import type { Project, AdminProject, Tag, SiteSettings } from "$lib/bindings";
```

- Never hand-edit files in `$lib/bindings/`.
- Request/mutation types (e.g., `CreateProjectData`) are defined in TypeScript alongside the API client, since they represent client intent, not server responses.
- Use `import type` for type-only imports (enforced by ESLint).

### Local Types

Types that exist only on the frontend (form state, UI state, component props) are defined alongside their consumers:

```typescript
// In the component or a co-located types.ts
interface ProjectFormState {
  name: string;
  slug: string;
  isDirty: boolean;
  errors: Map<string, string>;
}
```

## State Management

### Escalation Ladder

Start with the simplest option. Escalate only when needed.

1. **Component `$state`** — Local to one component. Default choice.
2. **Module runes (`.svelte.ts`)** — Shared across a few related components. Singleton class with `$state` properties.
3. **Context** — Subtree-scoped state via `setContext`/`getContext`. For layout-level concerns.
4. **Global stores** — App-wide singletons. Use sparingly (theme, auth).

### Class-Based Stores

For global/shared state, use the singleton class pattern with Svelte 5 runes:

```typescript
// lib/stores/theme.svelte.ts
class ThemeStore {
  isDark = $state<boolean>(true);

  toggle() {
    this.isDark = !this.isDark;
    this.persist();
  }

  private persist() {
    localStorage.setItem("theme", this.isDark ? "dark" : "light");
  }
}

export const themeStore = new ThemeStore();
```

- One class per concern.
- Exported as a singleton instance.
- Methods for mutations, `$state` for reactive fields.
- Persistence logic (localStorage, cookies) encapsulated in the class.

## Components

### When to Extract

Extract a component when any of these are true:

- It's reused in two or more places.
- It exceeds ~100 lines.
- It has a stable, describable interface.
- It represents a distinct UI concern (modal, form field, data table).

### Props

Use `$props()` with TypeScript interfaces:

```svelte
<script lang="ts">
  import type { AdminProject } from "$lib/bindings";

  interface Props {
    project: AdminProject;
    onDelete?: (id: string) => void;
  }

  let { project, onDelete }: Props = $props();
</script>
```

- Define a `Props` interface for non-trivial components.
- Use function props for callbacks (`onDelete`, `onSave`).
- Use `Snippet` for render-prop patterns (slot replacement in Svelte 5).

### Error Boundaries

Use `svelte:boundary` to catch render errors in isolated component subtrees:

```svelte
<svelte:boundary onerror={(error) => console.error("Render failed", error)}>
  <ProjectCard {project} />
  {#snippet failed(error)}
    <div class="text-red-500">Failed to render project card</div>
  {/snippet}
</svelte:boundary>
```

Use boundaries around:
- Third-party or complex components that might fail.
- Dynamic content where data shape isn't fully guaranteed.
- Non-critical UI sections where a failure shouldn't crash the page.

## Reactivity

### Svelte 5 Runes

Use runes exclusively. No legacy `$:` reactive statements.

| Rune | Use Case |
|---|---|
| `$state` | Mutable reactive value |
| `$derived` | Computed value (single expression) |
| `$derived.by` | Computed value (multi-statement) |
| `$effect` | Side effects (DOM manipulation, subscriptions) |
| `$props` | Component input declaration |

### Effect Discipline

`$effect` is for **side effects only** — DOM manipulation, event listeners, timers. Never use it for data transformation (use `$derived` instead).

```svelte
<script lang="ts">
  // Good: derived for data transformation
  const filteredProjects = $derived(
    projects.filter(p => p.status !== "hidden")
  );

  // Good: effect for DOM side effect
  $effect(() => {
    document.title = `${project.name} | xevion.dev`;
  });

  // Bad: effect for data transformation
  let filtered = $state<Project[]>([]);
  $effect(() => {
    filtered = projects.filter(p => p.status !== "hidden"); // Use $derived instead
  });
</script>
```

## Styling

### Tailwind v4

Tailwind is configured in `web/src/app.css` using the `@theme` directive. There is no `tailwind.config.js`.

```css
/* web/src/app.css */
@import "tailwindcss";

@theme {
  --color-bg-primary: var(--bg-primary);
  --font-sans: "Inter Variable", sans-serif;
  /* ... */
}
```

### Conventions

- Use Tailwind utility classes for styling. Avoid `<style>` blocks except for truly dynamic or complex CSS.
- Use the `cn()` helper (clsx + tailwind-merge) for conditional and composed class names.
- Use `dark:` prefix for dark mode variants.
- Keep color tokens semantic (`bg-primary`, `text-muted`) rather than literal (`bg-gray-900`).

```svelte
<div class={cn(
  "rounded-lg border p-4",
  isActive && "border-admin-accent",
  className,
)}>
```

## Logging

Use LogTape with category arrays:

```typescript
import { getLogger } from "@logtape/logtape";

const logger = getLogger(["ssr", "routes", "projects"]);
logger.info("Projects loaded", { count: projects.length });
```

- Category arrays enable hierarchical filtering (`ssr:routes:*`).
- Follow the same structured logging conventions as STYLE.md.
- `console.warn` / `console.error` acceptable for quick client-side diagnostics, but prefer LogTape for anything that runs on the server.

## Testing

### Vitest Setup

Unit and integration tests use Vitest. Test files are co-located with source:

```
lib/
├── api.ts
├── api.test.ts          ← API client tests
├── utils.ts
├── utils.test.ts        ← Utility function tests
└── components/
    ├── Button.svelte
    └── Button.test.ts   ← Component test
```

### API Client Tests

Test the Result-returning API functions with mocked fetch:

```typescript
import { describe, test, expect, vi } from "vitest";
import { getProjects } from "./api";

describe("getProjects", () => {
  test("returns Ok with project list on success", async () => {
    const mockProjects = [{ id: "1", name: "Test" }];
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(mockProjects),
    });

    const result = await getProjects();
    expect(result.isOk).toBe(true);
    result.match({
      Ok: (projects) => expect(projects).toEqual(mockProjects),
      Err: () => expect.unreachable("Should not be an error"),
    });
  });

  test("returns Err with ApiError on 404", async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 404,
      statusText: "Not Found",
    });

    const result = await getProjects();
    expect(result.isErr).toBe(true);
  });
});
```

### Component Tests

Use `@testing-library/svelte` for behavior-focused component tests:

```typescript
import { render, screen } from "@testing-library/svelte";
import ProjectCard from "./ProjectCard.svelte";

test("displays project name and status", () => {
  render(ProjectCard, {
    props: {
      project: { name: "My Project", status: "active", /* ... */ },
    },
  });

  expect(screen.getByText("My Project")).toBeInTheDocument();
  expect(screen.getByText("active")).toBeInTheDocument();
});
```

Test what users see and interact with, not implementation details.
