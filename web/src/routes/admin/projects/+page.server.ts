import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { renderIconsBatch } from "$lib/server/icons";
import type {
  AdminProject,
  ProjectStatus,
  TagWithIcon,
} from "$lib/admin-types";

export interface ProjectWithTagIcons extends Omit<AdminProject, "tags"> {
  tags: TagWithIcon[];
}

// Status display configuration (icons for server-side rendering)
const STATUS_ICONS: Record<ProjectStatus, string> = {
  active: "lucide:circle-check",
  maintained: "lucide:wrench",
  archived: "lucide:archive",
  hidden: "lucide:eye-off",
};

export const load: PageServerLoad = async ({ fetch }) => {
  const projects = await apiFetch<AdminProject[]>("/api/projects", { fetch });

  // Collect all icon identifiers for batch rendering
  const iconIds = new Set<string>();

  // Add status icons
  for (const icon of Object.values(STATUS_ICONS)) {
    iconIds.add(icon);
  }

  // Add tag icons
  for (const project of projects) {
    for (const tag of project.tags) {
      if (tag.icon) {
        iconIds.add(tag.icon);
      }
    }
  }

  // Batch render all icons
  const icons = await renderIconsBatch([...iconIds], { size: 12 });

  // Build status icons map
  const statusIcons: Record<ProjectStatus, string> = {
    active: icons.get(STATUS_ICONS.active) ?? "",
    maintained: icons.get(STATUS_ICONS.maintained) ?? "",
    archived: icons.get(STATUS_ICONS.archived) ?? "",
    hidden: icons.get(STATUS_ICONS.hidden) ?? "",
  };

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
    statusIcons,
  };
};
