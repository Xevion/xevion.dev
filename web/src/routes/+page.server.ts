import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { renderIconSVG } from "$lib/server/icons";
import type { AdminProject } from "$lib/admin-types";

export const load: PageServerLoad = async ({ fetch, parent }) => {
  // Get settings from parent layout
  const parentData = await parent();
  const settings = parentData.settings;

  const projects = await apiFetch<AdminProject[]>("/api/projects", { fetch });

  // Pre-render tag icons and clock icons (server-side only)
  const projectsWithIcons = await Promise.all(
    projects.map(async (project) => {
      const tagsWithIcons = await Promise.all(
        project.tags.map(async (tag) => ({
          ...tag,
          iconSvg: tag.icon
            ? (await renderIconSVG(tag.icon, { size: 12 })) || ""
            : "",
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

  // Pre-render social link icons (server-side only)
  const socialLinksWithIcons = await Promise.all(
    settings.socialLinks.map(async (link) => ({
      ...link,
      iconSvg: (await renderIconSVG(link.icon, { size: 16 })) || "",
    })),
  );

  return {
    projects: projectsWithIcons,
    socialLinksWithIcons,
  };
};
