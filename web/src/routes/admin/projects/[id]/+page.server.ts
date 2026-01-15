import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { collectTagIcons } from "$lib/server/tag-icons";
import type {
  AdminProject,
  AdminTagWithCount,
  AdminTag,
} from "$lib/admin-types";

export const load: PageServerLoad = async ({ params, fetch }) => {
  const { id } = params;

  // Fetch project and tags in parallel
  const [project, availableTags] = await Promise.all([
    apiFetch<AdminProject>(`/api/projects/${id}`, { fetch }).catch(() => null),
    apiFetch<AdminTagWithCount[]>("/api/tags", { fetch }),
  ]);

  // Collect icons for sprite (from available tags + project tags)
  const allTags: AdminTag[] = [...availableTags];
  if (project) {
    allTags.push(...project.tags);
  }
  const icons = await collectTagIcons(allTags);

  return {
    project,
    availableTags,
    icons,
  };
};
