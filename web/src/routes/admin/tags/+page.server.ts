import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import type { ApiTagWithCount } from "$lib/bindings";

export const load: PageServerLoad = async ({ fetch }) => {
  const result = await apiFetch<ApiTagWithCount[]>("/api/tags", { fetch });
  const tags = result.unwrapOr([]);

  // Sort by project count descending (popularity)
  const sortedTags = [...tags].sort((a, b) => b.projectCount - a.projectCount);

  return {
    tags: sortedTags,
  };
};
