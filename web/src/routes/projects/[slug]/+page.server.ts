import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { error } from "@sveltejs/kit";
import type { ApiProjectDetail } from "$lib/bindings";
import type { JSONContent } from "@tiptap/core";
import { renderDetailContent } from "$lib/tiptap/render.server";
import { getOGImageUrl } from "$lib/og-types";

export const load: PageServerLoad = async ({ params, fetch, url }) => {
  const result = await apiFetch<ApiProjectDetail>(
    `/api/projects/${params.slug}`,
    { fetch },
  );

  const project = result.unwrapOrElse((apiErr) => {
    throw error(apiErr.status || 404, apiErr.statusText);
  });

  // Every project has a detail page. Prose is optional — projects without
  // authored content render the hero/meta/links/related shell with no body.
  const rendered = project.detailContent
    ? await renderDetailContent(project.detailContent as JSONContent)
    : null;
  const html = rendered?.html ?? null;
  const toc = rendered?.toc ?? [];

  // The §NN section markers continue across the prose into the Gallery heading.
  // The prose h2s are numbered via CSS counters; the Gallery heading needs to
  // know how many preceded it, so count the h2s the renderer collected.
  const sectionCount = toc.filter((item) => item.level === 2).length;

  // Cache-bust the per-project OG card on every edit: the R2 object is overwritten
  // in place at og/project/{id}.png, so the URL must change for caches to refetch.
  const version = Date.parse(project.updatedAt);

  return {
    project,
    html,
    toc,
    sectionCount,
    metadata: {
      title: `${project.name} | Xevion`,
      description: project.shortDescription,
      ogImage: getOGImageUrl(
        { type: "project", id: project.id },
        Number.isNaN(version) ? undefined : version,
      ),
      url: `${url.origin}${url.pathname}`,
    },
  };
};
