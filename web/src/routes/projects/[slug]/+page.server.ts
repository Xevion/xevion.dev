import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { error } from "@sveltejs/kit";
import type { ApiProjectDetail, ApiAdminProject } from "$lib/bindings";
import type { JSONContent } from "@tiptap/core";
import { renderDetailContent } from "$lib/tiptap/render.server";

export const load: PageServerLoad = async ({ params, fetch }) => {
  const [result, listResult] = await Promise.all([
    apiFetch<ApiProjectDetail>(`/api/projects/${params.slug}`, { fetch }),
    apiFetch<ApiAdminProject[]>("/api/projects", { fetch }),
  ]);

  const project = result.unwrapOrElse((apiErr) => {
    throw error(apiErr.status || 404, apiErr.statusText);
  });

  // Every project has a detail page. Prose is optional — projects without
  // authored content render the hero/meta/links/related shell with no body.
  return {
    project,
    // The full ordered list drives the "Related projects" and Previous/Next pager.
    projects: listResult.unwrapOr([]),
    html: project.detailContent
      ? await renderDetailContent(project.detailContent as JSONContent)
      : null,
  };
};
