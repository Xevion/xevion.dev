import { error } from "@sveltejs/kit";
import { waitUntil } from "cloudflare:workers";
import { getAppLogger } from "#lib/logger.ts";
import { getOGImageUrl } from "#lib/og-types.ts";
import type { JSONContent } from "@tiptap/core";
import { prepareContent } from "#lib/server/content.ts";
import { renderDetailContent } from "#lib/tiptap/render.server.ts";
import { toApiProjectDetail } from "#lib/server/project-mapping.ts";
import { getProjectBundle } from "#lib/server/project-views.ts";
import { storeRenderedContent } from "#lib/server/projects.ts";
import type { TocItem } from "#lib/tiptap/toc.ts";
import type { PageServerLoad } from "./$types";

const logger = getAppLogger("routes", "project");

export const load: PageServerLoad = async ({ params, locals, url }) => {
  const bundle = await getProjectBundle(
    locals.db,
    params.slug,
    locals.user !== null,
  );
  if (!bundle) error(404, "Project not found");
  const { row } = bundle;

  let html = row.renderedHtml;
  let toc: TocItem[] = row.toc ?? [];
  if (row.detailContent != null && html == null) {
    try {
      const prepared = await prepareContent(row.detailContent);
      html = prepared.renderedHtml;
      toc = prepared.toc ?? [];
      waitUntil(
        storeRenderedContent(locals.db, row.id, prepared).catch(
          (cause: unknown) => {
            logger.warn("rendered content not stored", {
              projectId: row.id,
              cause: String(cause),
            });
          },
        ),
      );
    } catch (cause) {
      // Imported documents predate the stricter validator; render them as-is (output is still
      // sanitized) rather than showing an empty page until they are re-saved.
      logger.warn("stored content failed validation", {
        projectId: row.id,
        cause: String(cause),
      });
      const rendered = await renderDetailContent(
        row.detailContent as JSONContent,
      );
      html = rendered.html;
      toc = rendered.toc;
    }
  }

  // The prose is rendered server-side; the raw document never reaches the client.
  const project = toApiProjectDetail(
    { ...row, detailContent: null },
    bundle.tags,
    bundle.media,
    bundle.related,
  );

  // The §NN section markers continue across the prose into the Gallery heading.
  const sectionCount = toc.filter((item) => item.level === 2).length;

  // The per-project OG card is overwritten in place in R2, so the URL carries the
  // edit time to make caches refetch it.
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
        row.updatedAt.getTime(),
      ),
      imageAlt: `${project.name} preview`,
      url: `${url.origin}${url.pathname}`,
      type: "article" as const,
      publishedTime: project.createdAt,
      modifiedTime: project.updatedAt,
    },
  };
};
