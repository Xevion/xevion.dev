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
  icon: string | null;
  name: string;
  description: string;
  shortDescription: string;
  links: Link[];
  wakatimeOffset: number | null;
  technologies: Technology[] | null;
  bannerImage: string;
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
