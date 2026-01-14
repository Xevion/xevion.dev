import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { renderIconsBatch } from "$lib/server/icons";
import { addIconsToTags } from "$lib/server/tag-icons";
import { error } from "@sveltejs/kit";
import type { AdminTag, AdminProject } from "$lib/admin-types";

interface TagWithProjectsResponse {
  tag: AdminTag;
  projects: AdminProject[];
}

interface RelatedTagResponse extends AdminTag {
  cooccurrenceCount: number;
}

export interface TagPageData {
  tag: AdminTag & { iconSvg?: string };
  projects: AdminProject[];
  relatedTags: Array<RelatedTagResponse & { iconSvg?: string }>;
}

export const load: PageServerLoad = async ({ params, fetch }) => {
  const { slug } = params;

  // Fetch tag with projects
  let tagData: TagWithProjectsResponse;
  try {
    tagData = await apiFetch<TagWithProjectsResponse>(`/api/tags/${slug}`, {
      fetch,
    });
  } catch {
    throw error(404, "Tag not found");
  }

  // Fetch related tags
  let relatedTags: RelatedTagResponse[] = [];
  try {
    relatedTags = await apiFetch<RelatedTagResponse[]>(
      `/api/tags/${slug}/related`,
      { fetch },
    );
  } catch (err) {
    // Non-fatal - just show empty related tags
  }

  // Render main tag icon (single icon, just use renderIconsBatch directly)
  const iconIds = new Set<string>();
  if (tagData.tag.icon) {
    iconIds.add(tagData.tag.icon);
  }
  const icons = await renderIconsBatch([...iconIds], { size: 12 });

  const tagWithIcon = {
    ...tagData.tag,
    iconSvg: tagData.tag.icon
      ? (icons.get(tagData.tag.icon) ?? undefined)
      : undefined,
  };

  // Add icons to related tags using helper (preserving cooccurrenceCount)
  const relatedTagsWithIconsBase = await addIconsToTags(relatedTags);
  const relatedTagsWithIcons = relatedTags.map((tag, i) => ({
    ...relatedTagsWithIconsBase[i],
    cooccurrenceCount: tag.cooccurrenceCount,
  }));

  return {
    tag: tagWithIcon,
    projects: tagData.projects,
    relatedTags: relatedTagsWithIcons,
  } satisfies TagPageData;
};
