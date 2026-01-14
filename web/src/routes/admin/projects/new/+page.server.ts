import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { addIconsToTags } from "$lib/server/tag-icons";
import type { AdminTagWithCount } from "$lib/admin-types";

export const load: PageServerLoad = async ({ fetch }) => {
  const tagsWithCounts = await apiFetch<AdminTagWithCount[]>("/api/tags", {
    fetch,
  });

  // Add icons to tags
  const availableTags = await addIconsToTags(tagsWithCounts);

  return {
    availableTags,
  };
};
