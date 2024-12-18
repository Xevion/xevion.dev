import { createDirectus, rest } from "@directus/sdk";

const directus = createDirectus("https://api.xevion.dev").with(rest());

export default directus;
