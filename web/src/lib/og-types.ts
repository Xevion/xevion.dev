/**
 * Discriminated union of all OG image types.
 *
 * IMPORTANT: Keep in sync with Rust's OGImageSpec in src/og.rs
 */
export type OGImageSpec =
  | { type: "index" }
  | { type: "projects" }
  | { type: "project"; id: string };

/**
 * Generate the R2 public URL for an OG image.
 * Called at ISR/build time when generating page metadata.
 *
 * @param spec - The OG image specification
 * @returns Full URL to the R2-hosted image
 */
export function getOGImageUrl(spec: OGImageSpec): string {
  const R2_BASE = import.meta.env.VITE_OG_R2_BASE_URL;

  if (!R2_BASE) {
    // During prerendering or development, use a fallback placeholder
    return "/og/placeholder.png";
  }

  switch (spec.type) {
    case "index":
      return `${R2_BASE}/og/index.png`;
    case "projects":
      return `${R2_BASE}/og/projects.png`;
    case "project":
      return `${R2_BASE}/og/project/${spec.id}.png`;
  }
}
