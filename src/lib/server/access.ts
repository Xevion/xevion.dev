// Every route's access rule lives here; hooks.server.ts is the only caller. access.test.ts walks
// src/routes so a new route can't silently land on the wrong side of these rules.

export type Access =
  | { kind: "allow" }
  | { kind: "unauthorized" }
  | { kind: "not-found" }
  | { kind: "redirect"; to: string };

export interface AccessInput {
  pathname: string;
  method: string;
  authenticated: boolean;
  /** Only consulted for /setup, so the common path never pays for the lookup. */
  userExists: () => Promise<boolean>;
}

const ALLOW: Access = { kind: "allow" };

/** Admin pages reachable without a session. */
const PUBLIC_ADMIN_PAGES = new Set(["/admin/login"]);

/** API reads that expose admin-only data. */
const PRIVATE_API_READ_PREFIXES = [
  "/api/sessions",
  "/api/stats",
  "/api/github",
  "/api/session",
];

/** API writes an anonymous client may make. */
const PUBLIC_API_WRITES = new Set([
  "POST /api/login",
  "POST /api/logout",
  "POST /api/auth/device",
]);

export function isApiPath(pathname: string): boolean {
  return pathname === "/api" || pathname.startsWith("/api/");
}

function under(pathname: string, prefix: string): boolean {
  return pathname === prefix || pathname.startsWith(`${prefix}/`);
}

export async function resolveAccess(input: AccessInput): Promise<Access> {
  const { pathname, method, authenticated } = input;

  // Reached only through the cron wrapper, which authenticates with a shared secret in hooks.
  if (under(pathname, "/internal")) return { kind: "not-found" };

  if (pathname === "/setup") {
    return (await input.userExists())
      ? { kind: "redirect", to: "/admin/login" }
      : ALLOW;
  }

  if (under(pathname, "/admin")) {
    if (authenticated || PUBLIC_ADMIN_PAGES.has(pathname)) return ALLOW;
    return { kind: "redirect", to: "/admin/login" };
  }

  if (isApiPath(pathname)) {
    if (authenticated) return ALLOW;
    const read = method === "GET" || method === "HEAD" || method === "OPTIONS";
    if (read) {
      return PRIVATE_API_READ_PREFIXES.some((p) => under(pathname, p))
        ? { kind: "unauthorized" }
        : ALLOW;
    }
    return PUBLIC_API_WRITES.has(`${method} ${pathname}`)
      ? ALLOW
      : { kind: "unauthorized" };
  }

  return ALLOW;
}
