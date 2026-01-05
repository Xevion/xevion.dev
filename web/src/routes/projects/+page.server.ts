import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api";

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

export const load: PageServerLoad = async () => {
  const projects = await apiFetch<Project[]>("/api/projects");
  return {
    projects,
  };
};
