import type { ApiProjectLink } from "./ApiProjectLink";

export type ApiProject = {
  id: string;
  slug: string;
  name: string;
  shortDescription: string;
  links: Array<ApiProjectLink>;
};
