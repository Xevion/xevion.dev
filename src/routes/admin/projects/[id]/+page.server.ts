import type { PageServerLoad } from "./$types";
import { loadApi } from "#lib/server/admin-load.ts";
import type { ApiProjectDetail, ApiTagWithCount } from "#lib/bindings/index.ts";

export const load: PageServerLoad = async ({ params, fetch }) => {
  const [project, availableTags] = await Promise.all([
    loadApi<ApiProjectDetail>(
      fetch,
      `/api/projects/${encodeURIComponent(params.id)}`,
    ),
    loadApi<ApiTagWithCount[]>(fetch, "/api/tags"),
  ]);

  return { project, availableTags };
};
