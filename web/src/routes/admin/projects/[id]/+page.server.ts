import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { addIconsToTags } from "$lib/server/tag-icons";
import type { AdminProject, AdminTagWithCount } from "$lib/admin-types";

export const load: PageServerLoad = async ({ params, fetch }) => {
  const { id } = params;

  // Fetch project and tags in parallel
  const [project, tagsWithCounts] = await Promise.all([
    apiFetch<AdminProject>(`/api/projects/${id}`, { fetch }).catch(() => null),
    apiFetch<AdminTagWithCount[]>("/api/tags", { fetch }),
  ]);

  // Add icons to tags
  const availableTags = await addIconsToTags(tagsWithCounts);

  return {
    project,
    availableTags,
  };
};
