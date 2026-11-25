import type { PageServerLoad } from "./$types";
import { apiFetch } from "#lib/api.server.ts";
import type { ApiAdminProject } from "#lib/bindings/index.ts";

export const load: PageServerLoad = async ({ fetch }) => {
  const result = await apiFetch<ApiAdminProject[]>("/api/projects", {
    fetch,
  });

  return {
    projects: result.unwrapOr([]),
  };
};
