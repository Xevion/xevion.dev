import { env, waitUntil } from "cloudflare:workers";
import { apiError, notFound } from "./api.ts";
import { getAppLogger } from "#lib/logger.ts";
import { isIconSegment } from "#lib/types/icons.ts";

const logger = getAppLogger("icons");

const CACHE_CONTROL = "public, max-age=31536000, immutable";
const UPSTREAM = "https://api.iconify.design";

function svgResponse(body: BodyInit): Response {
  return new Response(body, {
    headers: {
      "content-type": "image/svg+xml",
      "cache-control": CACHE_CONTROL,
    },
  });
}

/** Serves an icon from R2, filling the bucket from Iconify on first use. */
export async function serveIcon(
  collection: string,
  name: string,
): Promise<Response> {
  if (!isIconSegment(collection) || !isIconSegment(name)) {
    return notFound("Icon");
  }
  const key = `icons/${collection}/${name}.svg`;

  const stored = await env.BUCKET.get(key);
  if (stored) return svgResponse(stored.body);

  const upstream = await fetch(`${UPSTREAM}/${collection}/${name}.svg`);
  if (upstream.status === 404) return notFound("Icon");
  if (!upstream.ok) {
    logger.warn("iconify request failed", {
      collection,
      name,
      status: upstream.status,
    });
    return apiError(502, "BAD_GATEWAY", "Icon source unavailable");
  }
  const svg = await upstream.text();
  // Iconify answers unknown names with a non-SVG body, sometimes with a 200.
  if (!svg.trimStart().startsWith("<svg")) return notFound("Icon");

  waitUntil(
    env.BUCKET.put(key, svg, {
      httpMetadata: {
        contentType: "image/svg+xml",
        cacheControl: CACHE_CONTROL,
      },
    }).catch((cause: unknown) => {
      logger.warn("icon cache write failed", { key, cause: String(cause) });
    }),
  );
  return svgResponse(svg);
}
