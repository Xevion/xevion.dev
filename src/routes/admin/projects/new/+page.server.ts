import type { PageServerLoad } from "./$types";
import { loadApi } from "#lib/server/admin-load.ts";
import type { ApiTagWithCount } from "#lib/bindings/index.ts";

export const load: PageServerLoad = async ({ fetch }) => ({
  availableTags: await loadApi<ApiTagWithCount[]>(fetch, "/api/tags"),
});
