// Presentation-layer derivations that bridge the API bindings to the redesign.
//
// The prototype dataset carried explicit `lang` / `featured` / tag-`kind` fields
// that the API does not. Rather than widen the backend, we derive them here from
// the data that already ships: tags (with slug + admin color), status, and dates.

import type { ApiTag } from "$lib/bindings";
import type { ProjectStatus } from "$lib/bindings";

/** Minimal shape shared by `ApiAdminProject` and `ApiProjectDetail`. */
export interface DisplayProject {
  slug: string;
  name: string;
  status: ProjectStatus;
  createdAt: string;
  lastActivity: string;
  tags: ApiTag[];
}

const NEUTRAL = "#71717a";

/**
 * Known languages keyed by tag slug, with the brand accent hues from the design.
 * Order is priority: a project's "primary" language is the first of these present
 * on it, so a Rust+Svelte+TS project reads as Rust.
 */
const LANGUAGES: { slug: string; color: string }[] = [
  { slug: "rust", color: "#b7410e" },
  { slug: "go", color: "#0a93b8" },
  { slug: "typescript", color: "#2f74c0" },
  { slug: "svelte", color: "#e8410a" },
  { slug: "react", color: "#0b93b9" },
  { slug: "next-js", color: "#3f3f46" },
  { slug: "nextjs", color: "#3f3f46" },
  { slug: "python", color: "#3572a5" },
  { slug: "javascript", color: "#c9a227" },
  { slug: "c", color: "#5f6b7a" },
  { slug: "cpp", color: "#5b63c4" },
];

const LANGUAGE_BY_SLUG = new Map(LANGUAGES.map((l) => [l.slug, l]));

/** Tags that count as part of the build stack ("Built with"), beyond languages. */
const TECH_SLUGS = new Set([
  "docker",
  "npm",
  "onnx",
  "sdl2",
  "emscripten",
  "webassembly",
  "webgpu",
  "discord",
  "mcp",
  "duckdb",
  "api",
  "sqlite",
  "postgres",
  "postgresql",
  "redis",
  "wasm",
]);

/** The project's primary language (name + accent), or null if none is tagged. */
export function detectLanguage(
  project: DisplayProject,
): { name: string; color: string } | null {
  for (const { slug, color } of LANGUAGES) {
    const tag = project.tags.find((t) => t.slug === slug);
    if (tag) return { name: tag.name, color };
  }
  return null;
}

/** Language accent for covers/marks; falls back to neutral grey. */
export function accentOf(project: DisplayProject): string {
  return detectLanguage(project)?.color ?? NEUTRAL;
}

/** Normalize an admin-stored color (with or without leading `#`) to a CSS hex. */
function normalizeColor(color: string | null | undefined): string | null {
  if (!color) return null;
  return color.startsWith("#") ? color : `#${color}`;
}

/**
 * The dot/cue color for a tag. Languages carry their brand hue; other tags use
 * their admin-set color when present, otherwise neutral grey.
 */
export function tagColor(tag: ApiTag): string {
  const lang = LANGUAGE_BY_SLUG.get(tag.slug);
  if (lang) return lang.color;
  return normalizeColor(tag.color) ?? NEUTRAL;
}

/** Whether a tag belongs in the "Built with" stack (a language or a known tech). */
export function isStackTag(tag: ApiTag): boolean {
  return LANGUAGE_BY_SLUG.has(tag.slug) || TECH_SLUGS.has(tag.slug);
}

/**
 * Featured = the two most-recently-active `active` projects. Drives the Hybrid
 * layout's pair of large cover cards. Returns a Set of featured slugs.
 */
export function featuredSlugs(
  projects: DisplayProject[],
  count = 2,
): Set<string> {
  const ranked = projects
    .filter((p) => p.status === "active")
    .slice()
    .sort(
      (a, b) =>
        new Date(b.lastActivity).getTime() - new Date(a.lastActivity).getTime(),
    )
    .slice(0, count);
  return new Set(ranked.map((p) => p.slug));
}

/** Status label + color (uppercase mono label in the UI, no dot). */
export function statusMeta(status: ProjectStatus): {
  label: string;
  color: string;
} {
  switch (status) {
    case "active":
      return { label: "Active", color: "#15803d" };
    case "maintained":
      return { label: "Maintained", color: "#0a8bc4" };
    case "archived":
      return { label: "Archived", color: NEUTRAL };
    case "hidden":
      return { label: "Hidden", color: NEUTRAL };
  }
}

/** Relative age, e.g. "3m ago" / "23h ago" / "9d ago" / "2mo ago". */
export function formatAge(dateString: string): string {
  const date = new Date(dateString);
  const diffMs = Date.now() - date.getTime();
  const diffMins = Math.floor(diffMs / (1000 * 60));
  const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  if (diffMins < 1) return "just now";
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffHours <= 48) return "yesterday";
  if (diffDays < 30) return `${diffDays}d ago`;
  if (diffDays < 365) return `${Math.floor(diffDays / 30)}mo ago`;
  return `${Math.floor(diffDays / 365)}y ago`;
}

/** Created month, e.g. "Mar 2026". */
export function formatCreated(dateString: string): string {
  return new Date(dateString).toLocaleDateString("en-US", {
    month: "short",
    year: "numeric",
  });
}
