import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import type { ApiAdminProject } from "$lib/bindings";

export const load: PageServerLoad = async ({ fetch }) => {
  const result = await apiFetch<ApiAdminProject[]>("/api/projects", {
    fetch,
  });

  return {
    projects: result.unwrapOr([]),
  };
};
