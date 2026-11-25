import { error } from "@sveltejs/kit";
import { waitUntil } from "cloudflare:workers";
import { getAppLogger } from "#lib/logger.ts";
import { getOGImageUrl } from "#lib/og-types.ts";
import { prepareContent } from "#lib/server/content.ts";
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
      logger.error("stored content failed to render", {
        projectId: row.id,
        cause: String(cause),
      });
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
