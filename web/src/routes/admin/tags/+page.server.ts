import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { addIconsToTags } from "$lib/server/tag-icons";
import type { AdminTagWithCount, TagWithIcon } from "$lib/admin-types";

export interface TagWithIconAndCount extends TagWithIcon {
  projectCount: number;
}

export const load: PageServerLoad = async ({ fetch }) => {
  const tags = await apiFetch<AdminTagWithCount[]>("/api/tags", { fetch });

  // Sort by project count descending (popularity)
  const sortedTags = [...tags].sort((a, b) => b.projectCount - a.projectCount);

  // Add icons to tags (type assertion safe - addIconsToTags preserves all properties)
  const tagsWithIcons = (await addIconsToTags(
    sortedTags,
  )) as TagWithIconAndCount[];

  return {
    tags: tagsWithIcons,
  };
};
