import type { PageServerLoad } from "./$types";

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
  // TODO: Fetch from Rust backend API
  return {
    projects: [] as Project[],
  };
};
