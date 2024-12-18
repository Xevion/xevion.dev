import { createDirectus, rest } from "@directus/sdk";

export interface Schema {
  metadata: Metadata;
}

export interface Metadata {
  tagline: string;
}

const directus = createDirectus<Schema>("https://api.xevion.dev").with(rest());

export default directus;
