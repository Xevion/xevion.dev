import { apiError, ok } from "#lib/server/api.ts";
import { listSessions } from "#lib/server/sessions.ts";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async ({ locals }) => {
  const user = locals.user;
  if (!user) apiError(401, "UNAUTHORIZED", "authentication required");
  return ok(await listSessions(locals.db, user.id, user.sessionId));
};
