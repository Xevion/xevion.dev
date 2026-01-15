import { json, error } from "@sveltejs/kit";
import type { RequestHandler } from "./$types";
import { getIconForApi } from "$lib/server/icons";

export const GET: RequestHandler = async (event) => {
  const { collection, name } = event.params;
  const identifier = `${collection}:${name}`;

  const iconData = await getIconForApi(identifier);

  if (!iconData) {
    throw error(404, `Icon not found: ${identifier}`);
  }

  return json(iconData);
};
