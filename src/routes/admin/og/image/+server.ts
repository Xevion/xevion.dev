import type { RequestHandler } from "./$types";
import { apiError } from "#lib/server/api.ts";
import { renderOgCard } from "#lib/server/og.ts";
import type { OGImageSpec } from "#lib/og-types.ts";

function parseSpec(params: URLSearchParams): OGImageSpec {
  const type = params.get("type");
  if (type === "index" || type === "projects") return { type };
  if (type === "project") {
    const id = params.get("id");
    if (!id) apiError(400, "VALIDATION_ERROR", 'Missing "id" for project card');
    return { type, id };
  }
  return apiError(400, "VALIDATION_ERROR", 'Missing or invalid "type"');
}

export const GET: RequestHandler = async ({ locals, url }) => {
  const png = await renderOgCard(locals.db, parseSpec(url.searchParams));
  return new Response(new Uint8Array(png), {
    headers: {
      "content-type": "image/png",
      "cache-control": "no-store",
    },
  });
};
