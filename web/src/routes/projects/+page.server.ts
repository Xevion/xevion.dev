import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api";
import { getOGImageUrl } from "$lib/og-types";

interface ProjectLink {
  url: string;
  title?: string;
}

export interface Project {
  id: string;
  name: string;
  shortDescription: string;
  icon?: string;
  links: ProjectLink[];
}

export const load: PageServerLoad = async ({ url }) => {
  const projects = await apiFetch<Project[]>("/api/projects");
  return {
    projects,
    metadata: {
      title: "Projects | Xevion.dev",
      description: "...",
      ogImage: getOGImageUrl({ type: "projects" }),
      url: url.toString(),
    },
  };
};
