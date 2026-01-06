import type { PageServerLoad } from "./$types";
import { MOCK_PROJECTS } from "$lib/mock-data/projects";
import { renderIconSVG } from "$lib/server/icons";

// import { apiFetch } from '$lib/api.server';
// import type { ApiProjectWithTags } from '$lib/admin-types';

export const load: PageServerLoad = async () => {
  // TODO: Replace with real API data
  // const projects = await apiFetch<ApiProjectWithTags[]>('/api/projects', { fetch });

  // Pre-render icon SVGs for tags (server-side only)
  const projectsWithIcons = await Promise.all(
    MOCK_PROJECTS.map(async (project) => {
      const tagsWithIcons = await Promise.all(
        project.tags.map(async (tag) => ({
          ...tag,
          iconSvg: (await renderIconSVG(tag.icon, { size: 12 })) || "",
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
