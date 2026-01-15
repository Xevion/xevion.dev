import { renderIconsBatch } from "./icons";
import type { AdminTag } from "$lib/admin-types";

/**
 * Collect and render icons from an array of tags.
 * Returns a record mapping icon identifiers to rendered SVG strings.
 *
 * @param tags - Array of tags to extract icons from
 * @returns Record of icon identifier to SVG string
 */
export async function collectTagIcons(
  tags: AdminTag[],
): Promise<Record<string, string>> {
  // Collect unique icon identifiers
  const iconIds = new Set<string>();
  for (const tag of tags) {
    if (tag.icon) {
      iconIds.add(tag.icon);
    }
  }

  // Return early if no icons
  if (iconIds.size === 0) {
    return {};
  }

  // Batch render all icons
  const iconsMap = await renderIconsBatch([...iconIds]);

  // Convert Map to plain object for serialization
  const icons: Record<string, string> = {};
  for (const [id, svg] of iconsMap) {
    icons[id] = svg;
  }

  return icons;
}
