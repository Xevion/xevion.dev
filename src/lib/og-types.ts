/**
 * Discriminated union of all OG image types.
 */
export type OGImageSpec =
  | { type: "index" }
  | { type: "projects" }
  | { type: "project"; id: string };

const MEDIA_BASE_URL = "https://media.xevion.dev";

/**
 * Generate the R2 public URL for an OG image.
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
  let url: string;
  switch (spec.type) {
    case "index":
      url = `${MEDIA_BASE_URL}/og/index.png`;
      break;
    case "projects":
      url = `${MEDIA_BASE_URL}/og/projects.png`;
      break;
    case "project":
      url = `${MEDIA_BASE_URL}/og/project/${spec.id}.png`;
      break;
  }

  return version != null ? `${url}?v=${version}` : url;
}
