import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { collectTagIcons } from "$lib/server/tag-icons";
import type { AdminProject } from "$lib/admin-types";

export const load: PageServerLoad = async ({ fetch }) => {
  const projects = await apiFetch<AdminProject[]>("/api/projects", { fetch });

  // Collect all tag icons across all projects
  const allTags = projects.flatMap((project) => project.tags);
  const icons = await collectTagIcons(allTags);

  return {
    projects,
    icons,
  };
};
