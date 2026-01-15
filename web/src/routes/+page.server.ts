import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { renderIconsBatch } from "$lib/server/icons";
import type { AdminProject } from "$lib/admin-types";

export const load: PageServerLoad = async ({ fetch, parent }) => {
  // Get settings from parent layout
  const parentData = await parent();
  const settings = parentData.settings;

  const projects = await apiFetch<AdminProject[]>("/api/projects", { fetch });

  // Collect all unique icon identifiers for batch rendering
  const iconIds = new Set<string>();

  // Collect tag icons
  for (const project of projects) {
    for (const tag of project.tags) {
      if (tag.icon) {
        iconIds.add(tag.icon);
      }
    }
  }

  // Collect social link icons
  for (const link of settings.socialLinks) {
    if (link.icon) {
      iconIds.add(link.icon);
    }
  }

  // Batch render all icons (single size, CSS handles scaling)
  const iconsMap = await renderIconsBatch([...iconIds]);

  // Convert Map to plain object for serialization
  const icons: Record<string, string> = {};
  for (const [id, svg] of iconsMap) {
    icons[id] = svg;
  }

  return {
    projects,
    icons,
    socialLinksWithIcons: settings.socialLinks,
  };
};
