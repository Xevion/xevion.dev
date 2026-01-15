import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { collectTagIcons } from "$lib/server/tag-icons";
import type { AdminTagWithCount } from "$lib/admin-types";

export const load: PageServerLoad = async ({ fetch }) => {
  const tags = await apiFetch<AdminTagWithCount[]>("/api/tags", { fetch });

  // Sort by project count descending (popularity)
  const sortedTags = [...tags].sort((a, b) => b.projectCount - a.projectCount);

  // Collect icons for sprite
  const icons = await collectTagIcons(sortedTags);

  return {
    tags: sortedTags,
    icons,
  };
};
