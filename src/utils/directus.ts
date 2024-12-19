import { createDirectus, rest } from "@directus/sdk";

export interface Schema {
  metadata: Metadata;
  project: Project[];
  technology: Technology[];
}

export interface Technology {
  id: string;
  name: string;
  url: string | null;
}

export interface Project {
  id: string;
  name: string;
  description: string;
  shortDescription: string;
  links: Link[];
  wakatime_offset: number | null;
  technologies: Technology[];
}

export interface Link {
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
