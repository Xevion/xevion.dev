import { error, redirect } from "@sveltejs/kit";
import type { Handle, HandleServerError } from "@sveltejs/kit/hooks";
import type { RequestEvent } from "@sveltejs/kit";
import { env, waitUntil } from "cloudflare:workers";
import { getDb } from "#lib/server/db/index.ts";
import { resolveSession } from "#lib/server/sessions.ts";
import { hasAnyUser } from "#lib/server/auth.ts";
import { isApiPath, resolveAccess } from "#lib/server/access.ts";
import { jsonifyErrorResponse } from "#lib/server/api.ts";
import {
  clearSessionCookie,
  SESSION_COOKIE,
} from "#lib/server/session-cookie.ts";
import { safeNextPath } from "#lib/redirects.ts";
import { getAppLogger } from "#lib/logger.ts";
import type { AuthUser } from "#lib/server/sessions.ts";

const logger = getAppLogger("http");

/** Hosts that answer with a permanent redirect to the canonical origin. */
const ALIAS_HOSTS = new Set(["walters.to", "www.walters.to", "www.xevion.dev"]);

export const CRON_SECRET_HEADER = "x-cron-secret";

async function resolveUser(
  event: RequestEvent,
): Promise<{ user: AuthUser | null; stale: boolean }> {
  const authorization = event.request.headers.get("authorization");
  const bearer = authorization?.startsWith("Bearer ")
    ? authorization.slice(7).trim()
    : undefined;
  const cookie = event.cookies.get(SESSION_COOKIE);
  const token = bearer ?? cookie;
  if (!token) return { user: null, stale: false };

  const resolved = await resolveSession(event.locals.db, token);
  if (resolved?.touch) {
    waitUntil(
      resolved.touch().catch((cause: unknown) => {
        logger.warn("session touch failed", { cause: String(cause) });
      }),
    );
  }
  return { user: resolved?.user ?? null, stale: !resolved && token === cookie };
}

function loginRedirect(event: RequestEvent): string {
  const target = `${event.url.pathname}${event.url.search}`;
  const next = safeNextPath(target);
  return next && next !== "/admin"
    ? `/admin/login?next=${encodeURIComponent(next)}`
    : "/admin/login";
}

function withSecurityHeaders(response: Response, request: Request): Response {
  const headers = new Headers(response.headers);
  headers.set("x-content-type-options", "nosniff");
  headers.set("referrer-policy", "strict-origin-when-cross-origin");
  headers.set("x-frame-options", "DENY");
  headers.set("permissions-policy", "geolocation=(), camera=(), microphone=()");
  // A tab resumed from bfcache after a redeploy would request chunk hashes that no longer exist.
  if (
    request.headers.get("sec-fetch-dest") === "document" &&
    !headers.has("cache-control")
  ) {
    headers.set("cache-control", "no-store");
  }
  return new Response(response.body, {
    status: response.status,
    statusText: response.statusText,
    headers,
  });
}

function earlyRedirect(url: URL): string | null {
  if (ALIAS_HOSTS.has(url.hostname))
    return `${env.SITE_ORIGIN}${url.pathname}${url.search}`;
  if (url.pathname.endsWith(".html") && !url.pathname.startsWith("/_app/")) {
    const stripped = url.pathname.slice(0, -".html".length);
    const path = stripped.endsWith("/index")
      ? stripped.slice(0, -"index".length)
      : stripped;
    return `${path.length > 1 ? path.replace(/\/$/, "") : "/"}${url.search}`;
  }
  return null;
}

// Single-admin site: once a user exists it always will, so the flag only ever flips to true.
let userExists = false;

export const handle: Handle = async ({ event, resolve }) => {
  const location = earlyRedirect(event.url);
  if (location)
    return new Response(null, { status: 301, headers: { location } });

  const isApi = isApiPath(event.url.pathname);

  event.locals.db = getDb(env.DB);

  const isCron =
    event.url.pathname.startsWith("/internal/") &&
    !!env.CRON_SECRET &&
    event.request.headers.get(CRON_SECRET_HEADER) === env.CRON_SECRET;

  if (isCron) {
    event.locals.user = null;
  } else {
    const { user, stale } = await resolveUser(event);
    event.locals.user = user;
    if (stale) clearSessionCookie(event.cookies);

    const access = await resolveAccess({
      pathname: event.url.pathname,
      method: event.request.method,
      authenticated: user !== null,
      userExists: async () => {
        if (!userExists) userExists = await hasAnyUser(event.locals.db);
        return userExists;
      },
    });
    if (access.kind === "unauthorized") {
      return Response.json(
        { message: "authentication required", code: "UNAUTHORIZED" },
        { status: 401 },
      );
    }
    if (access.kind === "not-found") error(404, "Not Found");
    if (access.kind === "redirect") {
      redirect(
        303,
        access.to === "/admin/login" ? loginRedirect(event) : access.to,
      );
    }
  }

  const resolved = await resolve(event);
  const response = isApi ? await jsonifyErrorResponse(resolved) : resolved;
  return withSecurityHeaders(response, event.request);
};

export const handleError: HandleServerError = ({
  kind,
  error: caught,
  event,
}) => {
  // App errors (error(...)) and framework errors (404s) already carry a safe status and message.
  if (kind !== "unknown") return;

  const errorId = crypto.randomUUID().slice(0, 8);
  logger.error("unhandled error", {
    errorId,
    route: event.route.id ?? event.url.pathname,
    method: event.request.method,
    error:
      caught instanceof Error
        ? `${caught.name}: ${caught.message}`
        : String(caught),
    stack: caught instanceof Error ? caught.stack : undefined,
  });
  return { message: "Internal Error", errorId, code: "INTERNAL" };
};
