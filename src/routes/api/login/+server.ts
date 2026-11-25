import { type } from "arktype";
import { env } from "cloudflare:workers";
import { apiError, ok, parseJsonBody } from "#lib/server/api.ts";
import { checkCredentials } from "#lib/server/auth.ts";
import { createSession } from "#lib/server/sessions.ts";
import { setSessionCookie } from "#lib/server/session-cookie.ts";
import { getAppLogger } from "#lib/logger.ts";
import type { RequestHandler } from "./$types";

const logger = getAppLogger("auth");

const LoginBody = type({ username: "string > 0", password: "string > 0" });

export const POST: RequestHandler = async ({
  request,
  locals,
  cookies,
  getClientAddress,
}) => {
  const body = await parseJsonBody(request, LoginBody);

  // Keyed by IP rather than username: there is one real account, so a username key buys nothing.
  const ip = getClientAddress();
  const { success } = await env.LOGIN_RATE_LIMITER.limit({ key: ip });
  if (!success) {
    logger.warn("login rate limited", { ip });
    apiError(
      429,
      "RATE_LIMITED",
      "Too many login attempts, try again in a minute",
    );
  }

  const user = await checkCredentials(locals.db, body.username, body.password);
  if (!user) {
    logger.warn("failed login", { username: body.username, ip });
    apiError(401, "UNAUTHORIZED", "Invalid username or password");
  }

  const { token, expiresAt } = await createSession(
    locals.db,
    user.id,
    "browser",
  );
  setSessionCookie(cookies, token, expiresAt);
  logger.info("login", { username: user.username, ip });
  return ok({ username: user.username });
};
