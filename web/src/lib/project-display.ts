// Presentation-layer derivations bridging the API bindings to the redesign.
//
// Accent and type are authored fields on the project (`accent_color`,
// `project_type`); there is no language/kind heuristic. Tags carry their own
// admin-set colors. Index card and row share one view-model so they can't drift.

import type { ApiTag, ApiAdminProject, ProjectStatus } from "$lib/bindings";

const NEUTRAL = "#71717a";

/** Normalize an admin-stored color (with or without leading `#`) to a CSS hex. */
function normalizeColor(color: string | null | undefined): string | null {
  if (!color) return null;
  return color.startsWith("#") ? color : `#${color}`;
}

/** The authored accent color, or neutral grey when unset. */
export function resolveAccent(accentColor: string | null | undefined): string {
  return normalizeColor(accentColor) ?? NEUTRAL;
}

/**
 * Black or white ink that stays legible on a solid `hex` fill (e.g. the demo
 * button painted with the author's accent). Uses WCAG relative luminance so a
 * light accent gets dark text instead of unreadable white.
 */
export function readableInk(hex: string): string {
  const m = (normalizeColor(hex) ?? NEUTRAL).slice(1);
  const v = (i: number) => parseInt(m.slice(i, i + 2), 16) / 255;
  const lin = (c: number) =>
    c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
  const luminance =
    0.2126 * lin(v(0)) + 0.7152 * lin(v(2)) + 0.0722 * lin(v(4));
  return luminance > 0.4 ? "#18181b" : "#ffffff";
}

/** A tag's dot/cue color: its admin-set color, else neutral grey. */
export function tagColor(tag: ApiTag): string {
  return normalizeColor(tag.color) ?? NEUTRAL;
}

/** Shared view-model for the index card + row, so the two can't drift. */
export function projectCardView(project: ApiAdminProject) {
  return {
    href: `/projects/${project.slug}`,
    accent: resolveAccent(project.accentColor),
    typeLabel: project.projectType ?? null,
    tags: project.tags.slice(0, 3),
  };
}

/** Minimal shape needed to rank for {@link featuredSlugs}. */
interface RankableProject {
  slug: string;
  status: ProjectStatus;
  lastActivity: string;
}

/**
 * Featured = the two most-recently-active `active` projects. Drives the Hybrid
 * layout's pair of large cover cards. Returns a Set of featured slugs.
 */
export function featuredSlugs(
  projects: RankableProject[],
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
  }
}

/** Created month, e.g. "Mar 2026". */
export function formatCreated(dateString: string): string {
  return new Date(dateString).toLocaleDateString("en-US", {
    month: "short",
    year: "numeric",
  });
}
