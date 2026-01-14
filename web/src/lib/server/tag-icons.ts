import { renderIconsBatch } from "./icons";
import type { AdminTag, TagWithIcon } from "$lib/admin-types";

/**
 * Add rendered icon SVG strings to tags by batch-rendering all icons
 *
 * @param tags - Array of tags to add icons to
 * @param options - Render options (size, etc.)
 * @returns Array of tags with iconSvg property populated
 */
export async function addIconsToTags(
  tags: AdminTag[],
  options?: { size?: number },
): Promise<TagWithIcon[]> {
  // Collect all icon identifiers
  const iconIds = new Set<string>();
  for (const tag of tags) {
    if (tag.icon) {
      iconIds.add(tag.icon);
    }
  }

  // Return early if no icons to render
  if (iconIds.size === 0) {
    return tags.map((tag) => ({ ...tag, iconSvg: undefined }));
  }

  // Batch render all icons
  const icons = await renderIconsBatch([...iconIds], {
    size: options?.size ?? 12,
  });

  // Map icons back to tags
  return tags.map((tag) => ({
    ...tag,
    iconSvg: tag.icon ? (icons.get(tag.icon) ?? undefined) : undefined,
  }));
}
