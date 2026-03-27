import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { error } from "@sveltejs/kit";
import type { ApiTag, ApiAdminProject } from "$lib/bindings";

interface TagWithProjectsResponse {
  tag: ApiTag;
  projects: ApiAdminProject[];
}

interface RelatedTagResponse extends ApiTag {
  cooccurrenceCount: number;
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
  } catch {
    // Non-fatal - just show empty related tags
  }

  return {
    tag: tagData.tag,
    projects: tagData.projects,
    relatedTags,
  };
};
