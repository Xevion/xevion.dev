import { env } from "cloudflare:workers";
import { listSitemapEntries } from "#lib/server/projects.ts";
import { renderSitemap } from "#lib/server/seo.ts";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async ({ locals }) =>
  new Response(
    renderSitemap(env.SITE_ORIGIN, await listSitemapEntries(locals.db)),
    {
      headers: {
        "content-type": "application/xml; charset=utf-8",
        "cache-control": "public, max-age=900, stale-while-revalidate=1800",
      },
    },
  );
