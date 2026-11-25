import { ok } from "#lib/server/api.ts";
import { revokeSession } from "#lib/server/sessions.ts";
import { clearSessionCookie } from "#lib/server/session-cookie.ts";
import type { RequestHandler } from "./$types";

export const POST: RequestHandler = async ({ locals, cookies }) => {
  if (locals.user)
    await revokeSession(locals.db, locals.user.id, locals.user.sessionId);
  clearSessionCookie(cookies);
  return ok(null);
};
