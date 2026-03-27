import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import type { ApiTagWithCount } from "$lib/bindings";

export const load: PageServerLoad = async ({ fetch }) => {
  const tags = await apiFetch<ApiTagWithCount[]>("/api/tags", { fetch });

  // Sort by project count descending (popularity)
  const sortedTags = [...tags].sort((a, b) => b.projectCount - a.projectCount);

  return {
    tags: sortedTags,
  };
};
