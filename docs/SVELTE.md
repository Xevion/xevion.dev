# SvelteKit Style Guide

Conventions for the SvelteKit 3 app (`src/`), which runs as a single Cloudflare Worker. Cross-cutting rules (imports, logging, errors, API shape) live in `STYLE.md`.

## Route Organization

Follow SvelteKit's file-based routing.

```
src/routes/
  +page.svelte, +page.server.ts     Homepage
  +layout.svelte, +layout.server.ts Root layout (session, settings)
  +error.svelte                     Error page
  projects/[slug]/                  Public project detail page
  admin/
    +layout.svelte                  Admin shell
    projects/, tags/, sessions/     Management pages
    settings/[[tab]]/               Settings with an optional tab param
    authorize/[id]/                 CLI device-flow approval
  api/                              +server.ts JSON endpoints
  internal/cron/                    Cron tick receiver (secret-gated)
  pgp/, keys/, resume/, sitemap.xml/, robots.txt/   Content and SEO endpoints
```

- One concern per route segment; group related routes under a directory.
- Access control is decided centrally in `src/lib/server/access.ts` (enforced by `src/hooks.server.ts`). Do not add auth guards in layouts or loads.
- Form submissions that are not JSON API calls (login setup, session revoke, CLI authorize) use SvelteKit form `actions` with `fail()` and `redirect()`.

## Data Loading

### Server Loads Only

Data loading uses `+page.server.ts` and `+layout.server.ts`, never universal `+page.ts`. Server loads are the only place that may touch D1 and `cloudflare:workers` bindings.

**Public pages** query D1 directly through `src/lib/server/*` functions, using `locals.db` and `locals.user`:

```typescript
export const load: PageServerLoad = async ({ locals, parent }) => {
  const [{ settings }, projects] = await Promise.all([
    parent(),
    listProjects(locals.db, locals.user !== null),
  ]);
  return { projects, socialLinks: settings.socialLinks };
};
```

**Admin pages** call the same-origin API through `loadApi` (`src/lib/server/admin-load.ts`), which unwraps the `{ data }` envelope or throws the API's error:

```typescript
export const load: PageServerLoad = async ({ fetch }) => ({
  projects: await loadApi<ApiAdminProject[]>(fetch, "/api/projects"),
});
```

### Errors in Loads

Loads return plain typed data; components never receive Results.

- **Detail pages:** throw `error(404, "Project not found")` when the record is missing.
- **Admin pages:** let `loadApi` throw on any failure, so admins see explicit errors.
- **Optional extras:** catch, log with `getAppLogger`, and degrade (see `src/routes/projects/[slug]/+page.server.ts`).

Use `refreshAll` instead of `invalidateAll` after mutations. External redirects need `{ external: true }`. Read page state through `$app/state` (`page`), never `$app/stores`.

## Browser API Client

Admin components call the API from the browser through two layers:

| Module                  | Role                                                                                                  |
| ----------------------- | ----------------------------------------------------------------------------------------------------- |
| `src/lib/api-client.ts` | `apiRequest<T>(url, method, body?, options?)`: fetch, unwrap `{ data }`, log, handle expired sessions |
| `src/lib/api.ts`        | One function per endpoint, grouped by resource (projects, tags, settings, sessions, media)            |

All functions return neverthrow `ResultAsync<T, ApiError>` and never throw. Callers branch on the result:

```typescript
const result = await updateAdminTag(data);
if (result.isErr()) {
  logger.error("Failed to update tag", { error: result.error });
  toast.error(result.error.message);
  return;
}
```

`ApiError` (`src/lib/errors.ts`) is a plain object `{ status, code, message, fieldErrors? }` with `isNotFound` and `isAuthError` helper functions. A 401 from any call navigates to `/admin/login` once, unless the call passes `redirectOnUnauthorized: false`.

## Type System

- API response types come from `#lib/bindings/index.ts` (hand-maintained; see `STYLE.md`).
- Request types for the admin client are in `src/lib/admin-types.ts`.
- Frontend-only types (form state, UI state, props) are defined next to their consumer.
- Route types come from `./$types`.

## State Management

Start with the simplest option and escalate only when needed.

1. **Component `$state`**: local to one component. The default.
2. **Module runes (`.svelte.ts`)**: shared across a few related components, such as `src/lib/components/project/toc-spy.svelte.ts`.
3. **Singleton class stores**: app-wide state in `src/lib/stores/` (`theme`, `auth`, `morph`).

```typescript
// src/lib/stores/theme.svelte.ts
class ThemeStore {
  isDark = $state<boolean>(true);

  toggle() {
    this.isDark = !this.isDark;
    localStorage.setItem("theme", this.isDark ? "dark" : "light");
    this.updateDOMClass();
  }
}

export const themeStore = new ThemeStore();
```

- One class per concern, exported as a singleton instance.
- Methods for mutations, `$state` fields for reactive data.
- Persistence (localStorage, sessionStorage) is encapsulated in the class and guarded with `typeof window` checks, since stores are also imported during SSR.

## Components

### When to Extract

Extract a component when it is reused, exceeds roughly 100 lines, has a stable describable interface, or is a distinct UI concern (modal, form field, table).

### Layout

- Shared components: `src/lib/components/`
- Admin-only primitives (`Button`, `Input`, `Modal`, `Table`, ...): `src/lib/components/admin/`
- Project-detail pieces: `src/lib/components/project/`

### Props

Use `$props()` with a `Props` interface. Callbacks are function props; render slots are `Snippet`s.

```svelte
<script lang="ts">
  import type { Snippet } from "svelte";
  import type { ApiAdminProject } from "#lib/bindings/index.ts";

  interface Props {
    project: ApiAdminProject;
    class?: string;
    onDelete?: (id: string) => void;
    children?: Snippet;
  }

  let { project, class: className, onDelete, children }: Props = $props();
</script>
```

Accept a `class` prop and merge it last with `cx` so callers can adjust layout. Use `$bindable()` only for true two-way inputs (`Input`, `ColorPicker`).

## Reactivity

Use runes exclusively. No legacy `$:` statements, `export let`, or `$$props`.

| Rune          | Use Case                              |
| ------------- | ------------------------------------- |
| `$state`      | Mutable reactive value                |
| `$derived`    | Computed value (single expression)    |
| `$derived.by` | Computed value (multi-statement)      |
| `$effect`     | Side effects (DOM, listeners, timers) |
| `$props`      | Component input declaration           |
| `$bindable`   | Two-way bound prop                    |

### Effect Discipline

`$effect` is for side effects only: DOM manipulation, event listeners, timers, loading data after mount. Never use it to transform data; use `$derived`.

```svelte
<script lang="ts">
  // Good
  const visible = $derived(projects.filter((p) => !p.hidden));

  // Good
  $effect(() => {
    document.title = `${project.name} | Xevion`;
  });

  // Bad: derive instead
  let filtered = $state<ApiAdminProject[]>([]);
  $effect(() => {
    filtered = projects.filter((p) => !p.hidden);
  });
</script>
```

Values that depend on the clock (relative ages) take an injectable `now` seeded by the server load, so SSR and hydration agree (`timeAgo` in `src/lib/time.ts`).

## Styling

Styling is PandaCSS. Never use Tailwind classes.

- Write styles with `css()` from `styled-system/css`; compose and merge with `cx()`. Layout helpers come from `styled-system/patterns` (`center`, `stack`, ...).
- Variant components use recipes from `styled-system/recipes` (see `src/lib/components/admin/Button.svelte`). Recipes used through wrapper components must declare `staticCss` for their variants, or only the defaults are emitted.
- Tokens and recipes are configured in `panda.config.ts`. Run `panda codegen` after changing it; `styled-system/` is generated.
- Dark mode uses the `_dark` condition: `css({ borderColor: "zinc.200", _dark: { borderColor: "zinc.800" } })`.
- Use semantic tokens (`surface`, `text.secondary`) over literal colors where one exists.
- Never put `:global()` inside `css()`; it is emitted literally and the browser drops the rule.
- Interactive elements need `cursor: pointer` in their base styles.
- A `<style>` block is acceptable only for what Panda cannot express: pseudo-element and state selectors on native elements, keyframes, and styles for third-party or `{@html}` content (see `src/lib/components/project/Lightbox.svelte`).

## Logging

Use `getAppLogger(...)` in server code and `getLogger(["admin", ...])` from `@logtape/logtape` in browser code, with a category array that mirrors the module path:

```typescript
const logger = getLogger(["admin", "components", "ProjectForm"]);
logger.error("Failed to save project", { error: result.error });
```

Messages are static; data goes in the properties object. See `STYLE.md` for levels and field names.

## Testing

Component tests are not set up. Keep logic worth testing in plain `.ts` modules (mappers, formatters, validators) and test those with the Vitest `unit` project. Database-backed logic is covered by `*.db.test.ts` files. See `STYLE.md` for the project layout.
