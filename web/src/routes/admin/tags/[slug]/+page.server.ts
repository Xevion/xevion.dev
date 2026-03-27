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
  const tagResult = await apiFetch<TagWithProjectsResponse>(
    `/api/tags/${slug}`,
    { fetch },
  );
  const tagData = tagResult.unwrapOrElse((apiErr) => {
    throw error(apiErr.status || 404, apiErr.statusText || "Tag not found");
  });

  // Fetch related tags
  const relatedResult = await apiFetch<RelatedTagResponse[]>(
    `/api/tags/${slug}/related`,
    { fetch },
  );
  const relatedTags = relatedResult.unwrapOr([]);

  return {
    tag: tagData.tag,
    projects: tagData.projects,
    relatedTags,
  };
};
