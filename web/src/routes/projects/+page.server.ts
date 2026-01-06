import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { getOGImageUrl } from "$lib/og-types";
import { renderIconSVG } from "$lib/server/icons";

interface ProjectLink {
  url: string;
  title?: string;
}

export interface Project {
  id: string;
  slug: string;
  name: string;
  shortDescription: string;
  icon?: string;
  iconSvg?: string;
  links: ProjectLink[];
}

export const load: PageServerLoad = async ({ url }) => {
  const projects = await apiFetch<Project[]>("/api/projects");

  // Render icon SVGs server-side
  const projectsWithIcons = await Promise.all(
    projects.map(async (project) => ({
      ...project,
      iconSvg: await renderIconSVG(project.icon ?? "lucide:heart", {
        class: "text-3xl opacity-80 saturate-0",
      }),
    })),
  );

  return {
    projects: projectsWithIcons,
    metadata: {
      title: "Projects | Xevion.dev",
      description: "...",
      ogImage: getOGImageUrl({ type: "projects" }),
      url: url.toString(),
    },
  };
};
