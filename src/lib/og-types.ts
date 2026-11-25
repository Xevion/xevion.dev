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
 * @param version - Optional cache-bust token (e.g. a project's `updatedAt`).
 *   Appended as `?v=`, so the long-lived `immutable` R2 object refreshes the
 *   instant the underlying content changes (the key is overwritten in place).
 * @returns Full URL to the R2-hosted image
 */
export function getOGImageUrl(
  spec: OGImageSpec,
  version?: string | number,
): string {
  const R2_BASE = import.meta.env.VITE_OG_R2_BASE_URL;

  if (!R2_BASE) {
    // During prerendering or development, use a fallback placeholder
    return "/og/placeholder.png";
  }

  let url: string;
  switch (spec.type) {
    case "index":
      url = `${R2_BASE}/og/index.png`;
      break;
    case "projects":
      url = `${R2_BASE}/og/projects.png`;
      break;
    case "project":
      url = `${R2_BASE}/og/project/${spec.id}.png`;
      break;
  }

  return version != null ? `${url}?v=${version}` : url;
}
