import type { PageServerLoad } from "./$types";
import { loadApi } from "#lib/server/admin-load.ts";
import type { ApiTagWithCount } from "#lib/bindings/index.ts";

export const load: PageServerLoad = async ({ fetch }) => {
  const tags = await loadApi<ApiTagWithCount[]>(fetch, "/api/tags");

  return {
    tags: [...tags].sort((a, b) => b.projectCount - a.projectCount),
  };
};
