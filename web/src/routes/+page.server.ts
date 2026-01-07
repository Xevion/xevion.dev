import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { renderIconSVG } from "$lib/server/icons";
import type { AdminProject } from "$lib/admin-types";

export const load: PageServerLoad = async ({ fetch }) => {
  const projects = await apiFetch<AdminProject[]>("/api/projects", { fetch });

  // Pre-render tag icons and clock icons (server-side only)
  const projectsWithIcons = await Promise.all(
    projects.map(async (project) => {
      const tagsWithIcons = await Promise.all(
        project.tags.map(async (tag) => ({
          ...tag,
          iconSvg: tag.icon ? (await renderIconSVG(tag.icon, { size: 12 })) || "" : "",
        })),
      );

      const clockIconSvg =
        (await renderIconSVG("lucide:clock", { size: 12 })) || "";

      return {
        ...project,
        tags: tagsWithIcons,
        clockIconSvg,
      };
    }),
  );

  return {
    projects: projectsWithIcons,
  };
};
