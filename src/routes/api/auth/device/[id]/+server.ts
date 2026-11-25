import { notFound, ok } from "#lib/server/api.ts";
import { pollDeviceAuth } from "#lib/server/device-auth.ts";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async ({ params, locals }) => {
  const result = await pollDeviceAuth(locals.db, params.id);
  if (result.status === "unknown") notFound("Device request");
  return ok(result, {
    headers: { "cache-control": "no-store" },
  });
};
