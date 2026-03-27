import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import type { ApiAdminProject, ApiTagWithCount } from "$lib/bindings";

export const load: PageServerLoad = async ({ params, fetch }) => {
  const { id } = params;

  // Fetch project and tags in parallel
  const [project, availableTags] = await Promise.all([
    apiFetch<ApiAdminProject>(`/api/projects/${id}`, { fetch }).catch(
      () => null,
    ),
    apiFetch<ApiTagWithCount[]>("/api/tags", { fetch }),
  ]);

  return {
    project,
    availableTags,
  };
};
