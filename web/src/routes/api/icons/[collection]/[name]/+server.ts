import { json, error } from "@sveltejs/kit";
import type { RequestHandler } from "./$types";
import { requireAuth } from "$lib/server/auth";
import { getIcon } from "$lib/server/icons";

export const GET: RequestHandler = async (event) => {
  // Require authentication
  requireAuth(event);

  const { collection, name } = event.params;
  const identifier = `${collection}:${name}`;

  const iconData = await getIcon(identifier);

  if (!iconData) {
    throw error(404, `Icon not found: ${identifier}`);
  }

  return json(iconData);
};
