import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import type { AdminTagWithCount } from "$lib/admin-types";

export const load: PageServerLoad = async ({ fetch }) => {
  const tags = await apiFetch<AdminTagWithCount[]>("/api/tags", { fetch });

  // Sort by project count descending (popularity)
  const sortedTags = [...tags].sort((a, b) => b.projectCount - a.projectCount);

  return {
    tags: sortedTags,
  };
};
