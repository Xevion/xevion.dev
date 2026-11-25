import { apiError, ok } from "#lib/server/api.ts";
import { getSessionExpiry } from "#lib/server/sessions.ts";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async ({ locals }) => {
  const user = locals.user;
  if (!user) apiError(401, "UNAUTHORIZED", "authentication required");
  const expiresAt = await getSessionExpiry(locals.db, user.sessionId);
  return ok({
    username: user.username,
    sessionType: user.sessionKind,
    expiresAt: expiresAt?.toISOString() ?? null,
  });
};
