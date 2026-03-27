import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { error } from "@sveltejs/kit";
import type { ApiAdminProject, ApiTagWithCount } from "$lib/bindings";

export const load: PageServerLoad = async ({ params, fetch }) => {
  const { id } = params;

  // Fetch project and tags in parallel
  const [projectResult, tagsResult] = await Promise.all([
    apiFetch<ApiAdminProject>(`/api/projects/${id}`, { fetch }),
    apiFetch<ApiTagWithCount[]>("/api/tags", { fetch }),
  ]);

  const project = projectResult.unwrapOrElse((apiErr) => {
    throw error(apiErr.status || 404, apiErr.statusText);
  });

  return {
    project,
    availableTags: tagsResult.unwrapOr([]),
  };
};
