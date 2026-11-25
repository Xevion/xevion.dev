import type { PageServerLoad } from "./$types";
import { loadApi } from "#lib/server/admin-load.ts";
import type {
  ApiAdminProject,
  ApiRelatedTag,
  ApiTag,
} from "#lib/bindings/index.ts";

export const load: PageServerLoad = async ({ params, fetch }) => {
  const slug = encodeURIComponent(params.slug);

  const [tagData, relatedTags] = await Promise.all([
    loadApi<{ tag: ApiTag; projects: ApiAdminProject[] }>(
      fetch,
      `/api/tags/${slug}`,
    ),
    loadApi<ApiRelatedTag[]>(fetch, `/api/tags/${slug}/related`),
  ]);

  return {
    tag: tagData.tag,
    projects: tagData.projects,
    relatedTags,
  };
};
