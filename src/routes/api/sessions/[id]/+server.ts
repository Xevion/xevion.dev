import { apiError, notFound, ok } from "#lib/server/api.ts";
import { revokeSession } from "#lib/server/sessions.ts";
import { clearSessionCookie } from "#lib/server/session-cookie.ts";
import type { RequestHandler } from "./$types";

export const DELETE: RequestHandler = async ({ params, locals, cookies }) => {
  const user = locals.user;
  if (!user) apiError(401, "UNAUTHORIZED", "authentication required");
  if (!(await revokeSession(locals.db, user.id, params.id)))
    notFound("Session");
  if (params.id === user.sessionId) clearSessionCookie(cookies);
  return ok(null);
};
