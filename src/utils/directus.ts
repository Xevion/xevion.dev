import { env } from "@/env/server.mjs";
import { createDirectus, rest, staticToken } from "@directus/sdk";

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

  // core fields
  date_created: string;
  date_updated: string;
  sort: number; // used for ordering
  status: string;

  // relationships
  links: number[] | ProjectLink[]; // One2Many
  technologies: number[] | ProjectTechnology[]; // Many2Many

  // relevant fields
  icon: string | null;
  name: string;
  description: string;
  shortDescription: string;

  // misc fields
  featured: boolean; // places the project in the 'featured' section
  autocheckUpdated: boolean; // triggers a cron job to check for updates
  wakatimeOffset: number | null; // offsets the WakaTime fetched data
  bannerImage: string; // file identifier
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

const directus = createDirectus<Schema>("https://api.xevion.dev", {
  globals: {
    fetch: (input, init) => {
      console.log(`${init.method?.toUpperCase()} ${input}`);
      return fetch(input, init);
    },
  },
})
  .with(staticToken(env.DIRECTUS_API_TOKEN))
  .with(rest());

export default directus;
