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

  // Every project has a detail page. Prose is optional — projects without
  // authored content render the hero/meta/links/related shell with no body.
  const html = project.detailContent
    ? await renderDetailContent(project.detailContent as JSONContent)
    : null;

  // The §NN section markers continue across the prose into the Gallery heading.
  // The prose h2s are numbered via CSS counters; the Gallery heading needs to
  // know how many preceded it, so count them here from the rendered HTML.
  const sectionCount = html ? (html.match(/<h2/g) ?? []).length : 0;

  return { project, html, sectionCount };
};
