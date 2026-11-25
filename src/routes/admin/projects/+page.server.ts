import type { PageServerLoad } from "./$types";
import { loadApi } from "#lib/server/admin-load.ts";
import type { ApiAdminProject } from "#lib/bindings/index.ts";

export const load: PageServerLoad = async ({ fetch }) => ({
  projects: await loadApi<ApiAdminProject[]>(fetch, "/api/projects"),
});
