import { env } from "cloudflare:workers";
import { renderRobots } from "#lib/server/seo.ts";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = () =>
  new Response(renderRobots(env.SITE_ORIGIN), {
    headers: {
      "content-type": "text/plain; charset=utf-8",
      "cache-control": "public, max-age=86400",
    },
  });
