import { type } from "arktype";
import { env } from "cloudflare:workers";
import { apiError, ok, parseJsonBody } from "#lib/server/api.ts";
import {
  MAX_PASSWORD_LENGTH,
  MIN_PASSWORD_LENGTH,
  changePassword,
} from "#lib/server/auth.ts";
import { getAppLogger } from "#lib/logger.ts";
import type { RequestHandler } from "./$types";

const logger = getAppLogger("auth");

const PasswordBody = type({
  currentPassword: "string > 0",
  newPassword: `${MIN_PASSWORD_LENGTH} <= string <= ${MAX_PASSWORD_LENGTH}`,
});

export const POST: RequestHandler = async ({
  request,
  locals,
  getClientAddress,
}) => {
  const user = locals.user;
  if (!user) apiError(401, "UNAUTHORIZED", "Authentication required");
  const body = await parseJsonBody(request, PasswordBody);

  // A stolen CLI token must not be able to brute-force the current password.
  const ip = getClientAddress();
  const { success } = await env.LOGIN_RATE_LIMITER.limit({ key: ip });
  if (!success) {
    apiError(429, "RATE_LIMITED", "Too many attempts, try again in a minute");
  }

  const result = await changePassword(
    locals.db,
    user.id,
    user.sessionId,
    body.currentPassword,
    body.newPassword,
  );
  if (!result.changed) {
    logger.warn("password change rejected", { username: user.username, ip });
    apiError(403, "FORBIDDEN", "Current password is incorrect", {
      currentPassword: "Incorrect password",
    });
  }

  logger.info("password changed", {
    username: user.username,
    revokedSessions: result.revokedSessions,
  });
  return ok({ revokedSessions: result.revokedSessions });
};
