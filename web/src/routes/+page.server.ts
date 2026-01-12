import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { renderIconsBatch } from "$lib/server/icons";
import type { AdminProject } from "$lib/admin-types";

const CLOCK_ICON = "lucide:clock";

export const load: PageServerLoad = async ({ fetch, parent }) => {
  // Get settings from parent layout
  const parentData = await parent();
  const settings = parentData.settings;

  const projects = await apiFetch<AdminProject[]>("/api/projects", { fetch });

  // Collect all icon identifiers for batch rendering
  const smallIconIds = new Set<string>();
  const largeIconIds = new Set<string>();

  // Add static icons
  smallIconIds.add(CLOCK_ICON);

  // Collect tag icons (size 12)
  for (const project of projects) {
    for (const tag of project.tags) {
      if (tag.icon) {
        smallIconIds.add(tag.icon);
      }
    }
  }

  // Collect social link icons (size 16)
  for (const link of settings.socialLinks) {
    if (link.icon) {
      largeIconIds.add(link.icon);
    }
  }

  // Batch render all icons (two batches for different sizes)
  const [smallIcons, largeIcons] = await Promise.all([
    renderIconsBatch([...smallIconIds], { size: 12 }),
    renderIconsBatch([...largeIconIds], { size: 16 }),
  ]);

  // Map icons back to projects
  const projectsWithIcons = projects.map((project) => ({
    ...project,
    tags: project.tags.map((tag) => ({
      ...tag,
      iconSvg: tag.icon ? (smallIcons.get(tag.icon) ?? "") : "",
    })),
    clockIconSvg: smallIcons.get(CLOCK_ICON) ?? "",
  }));

  // Map icons back to social links
  const socialLinksWithIcons = settings.socialLinks.map((link) => ({
    ...link,
    iconSvg: largeIcons.get(link.icon) ?? "",
  }));

  return {
    projects: projectsWithIcons,
    socialLinksWithIcons,
  };
};
