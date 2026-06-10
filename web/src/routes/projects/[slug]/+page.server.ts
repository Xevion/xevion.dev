import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { error } from "@sveltejs/kit";
import type { ApiProjectDetail } from "$lib/bindings";
import type { JSONContent } from "@tiptap/core";
import { renderDetailContent } from "$lib/tiptap/render.server";

export const load: PageServerLoad = async ({ params, fetch }) => {
  const result = await apiFetch<ApiProjectDetail>(
    `/api/projects/${params.slug}`,
    { fetch },
  );

  const project = result.unwrapOrElse((apiErr) => {
    throw error(apiErr.status || 404, apiErr.statusText);
  });

  // Only projects with authored content get a detail page; everything else 404s
  // (cards for those link straight to demo/GitHub and never point here).
  if (!project.detailContent) {
    throw error(404, "Not Found");
  }

  return {
    project,
    html: renderDetailContent(project.detailContent as JSONContent),
  };
};
