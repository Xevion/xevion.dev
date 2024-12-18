import { createDirectus, rest } from "@directus/sdk";

export type Schema = {
  metadata: Metadata[];
};

export type Metadata = {
  tagline: string;
};

const directus = createDirectus<Schema>("https://api.xevion.dev").with(rest());

export default directus;
