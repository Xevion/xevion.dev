import { json } from "@sveltejs/kit";
import type { RequestHandler } from "./$types";
import { requireAuth } from "#lib/server/auth.ts";
import { getCollections } from "#lib/server/icons.ts";

export const GET: RequestHandler = async (event) => {
  // Require authentication
  requireAuth(event);

  const collections = await getCollections();

  return json({
    collections,
    count: collections.length,
  });
};
