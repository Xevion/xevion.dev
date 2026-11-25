import type { RequestHandler } from "./$types";
import { notFound } from "#lib/server/api.ts";
import { serveIcon } from "#lib/server/icons.ts";

export const GET: RequestHandler = ({ params }) => {
  if (!params.file.endsWith(".svg")) return notFound("Icon");
  return serveIcon(params.collection, params.file.slice(0, -".svg".length));
};
