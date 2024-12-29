import { createDirectus, rest } from "@directus/sdk";

export interface Schema {
  metadata: Metadata;
  project: Project[];
  technology: Technology[];
  link: Link[];
  project_technology: ProjectTechnology[];
  project_link: ProjectLink[];
}

export interface Technology {
  id: string;
  name: string;
  url: string | null;
}

export interface ProjectTechnology {
  id: string;
  project_id: string;
  technology_id: string;
}

export interface Project {
  id: string;
  
  // One2Many
  links: number[] | ProjectLink[];
  // Many2Many
  technologies: number[] | ProjectTechnology[];

  icon: string | null;
  name: string;
  description: string;
  shortDescription: string;

  featured: boolean;
  wakatimeOffset: number | null;
  bannerImage: string;
}

export interface Link {
  id: string;
  project_id: string;
  icon: string;
  url: string;
  description: string | null;
}

export interface ProjectLink {
  id: string;
  project_id: string;
  sort: number;
  icon: string;
  url: string;
  description: string | null;
}

export interface Metadata {
  tagline: string;
  resume: string;
  resumeFilename: string;
}

const directus = createDirectus<Schema>("https://api.xevion.dev").with(rest());

export default directus;
