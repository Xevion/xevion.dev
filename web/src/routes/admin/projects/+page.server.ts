import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { renderIconsBatch } from "$lib/server/icons";
import type { AdminProject, TagWithIcon } from "$lib/admin-types";

export interface ProjectWithTagIcons extends Omit<AdminProject, "tags"> {
  tags: TagWithIcon[];
}

export const load: PageServerLoad = async ({ fetch }) => {
  const projects = await apiFetch<AdminProject[]>("/api/projects", { fetch });

  // Collect all tag icon identifiers for batch rendering
  const iconIds = new Set<string>();
  for (const project of projects) {
    for (const tag of project.tags) {
      if (tag.icon) {
        iconIds.add(tag.icon);
      }
    }
  }

  // Batch render all icons
  const icons = await renderIconsBatch([...iconIds], { size: 12 });

  // Map icons back to project tags
  const projectsWithIcons: ProjectWithTagIcons[] = projects.map((project) => ({
    ...project,
    tags: project.tags.map((tag) => ({
      ...tag,
      iconSvg: tag.icon ? (icons.get(tag.icon) ?? undefined) : undefined,
    })),
  }));

  return {
    projects: projectsWithIcons,
  };
};
