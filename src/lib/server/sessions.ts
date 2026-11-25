import { and, desc, eq, lt } from "drizzle-orm";
import type { Db } from "./db/index.ts";
import { sessions, users } from "./db/schema.ts";
import { randomToken, sha256Hex } from "./crypto.ts";
import type { ApiSession } from "#lib/bindings/index.ts";

// Kept free of $app/* so workerd db tests can import it without the SvelteKit plugin.

export type SessionKind = (typeof sessions.$inferSelect)["kind"];

export interface AuthUser {
  id: number;
  username: string;
  sessionId: string;
  sessionKind: SessionKind;
}

const HOUR = 3600_000;
const DAY = 24 * HOUR;
export const BROWSER_SESSION_TTL_MS = 7 * DAY;
export const CLI_SESSION_TTL_MS = 90 * DAY;
// CLI sessions slide, but rewriting the row on every request is waste.
const CLI_SLIDE_INTERVAL_MS = 6 * HOUR;
const ACTIVITY_INTERVAL_MS = 5 * 60_000;

export const CLI_TOKEN_PREFIX = "xev_";

export async function createSession(
  db: Db,
  userId: number,
  kind: SessionKind,
  label?: string,
): Promise<{ token: string; expiresAt: Date; id: string }> {
  const token =
    kind === "cli" ? `${CLI_TOKEN_PREFIX}${randomToken(30)}` : randomToken(32);
  const now = Date.now();
  const expiresAt = new Date(
    now + (kind === "cli" ? CLI_SESSION_TTL_MS : BROWSER_SESSION_TTL_MS),
  );
  const [row] = await db
    .insert(sessions)
    .values({
      userId,
      tokenHash: await sha256Hex(token),
      kind,
      label: label ?? null,
      expiresAt,
      lastActiveAt: new Date(now),
    })
    .returning({ id: sessions.id });
  return { token, expiresAt, id: row!.id };
}

interface ResolvedSession {
  user: AuthUser;
  /** Deferred bookkeeping for the caller to run under waitUntil, or null when nothing is due. */
  touch: (() => Promise<unknown>) | null;
}

export async function resolveSession(
  db: Db,
  token: string,
): Promise<ResolvedSession | null> {
  const tokenHash = await sha256Hex(token);
  const row = await db
    .select({
      id: sessions.id,
      kind: sessions.kind,
      expiresAt: sessions.expiresAt,
      lastActiveAt: sessions.lastActiveAt,
      userId: users.id,
      username: users.username,
    })
    .from(sessions)
    .innerJoin(users, eq(sessions.userId, users.id))
    .where(eq(sessions.tokenHash, tokenHash))
    .get();
  if (!row) return null;

  const now = Date.now();
  if (row.expiresAt.getTime() <= now) {
    await db.delete(sessions).where(eq(sessions.id, row.id));
    return null;
  }

  const set: Partial<typeof sessions.$inferInsert> = {};
  if (now - row.lastActiveAt.getTime() > ACTIVITY_INTERVAL_MS)
    set.lastActiveAt = new Date(now);
  if (
    row.kind === "cli" &&
    row.expiresAt.getTime() - now < CLI_SESSION_TTL_MS - CLI_SLIDE_INTERVAL_MS
  ) {
    set.expiresAt = new Date(now + CLI_SESSION_TTL_MS);
  }

  return {
    user: {
      id: row.userId,
      username: row.username,
      sessionId: row.id,
      sessionKind: row.kind,
    },
    touch:
      Object.keys(set).length > 0
        ? () => db.update(sessions).set(set).where(eq(sessions.id, row.id))
        : null,
  };
}

export async function revokeSession(
  db: Db,
  userId: number,
  sessionId: string,
): Promise<boolean> {
  const deleted = await db
    .delete(sessions)
    .where(and(eq(sessions.id, sessionId), eq(sessions.userId, userId)))
    .returning({ id: sessions.id });
  return deleted.length > 0;
}

export async function listSessions(
  db: Db,
  userId: number,
  currentSessionId: string,
): Promise<ApiSession[]> {
  const rows = await db
    .select()
    .from(sessions)
    .where(eq(sessions.userId, userId))
    .orderBy(desc(sessions.lastActiveAt));
  const now = Date.now();
  return rows
    .filter((r) => r.expiresAt.getTime() > now)
    .map((r) => ({
      id: r.id,
      sessionType: r.kind,
      label: r.label ?? undefined,
      createdAt: r.createdAt.toISOString(),
      lastActiveAt: r.lastActiveAt.toISOString(),
      expiresAt: r.expiresAt.toISOString(),
      current: r.id === currentSessionId,
    }));
}

export async function deleteExpiredSessions(db: Db): Promise<number> {
  const deleted = await db
    .delete(sessions)
    .where(lt(sessions.expiresAt, new Date()))
    .returning({ id: sessions.id });
  return deleted.length;
}

export async function getSessionExpiry(
  db: Db,
  sessionId: string,
): Promise<Date | null> {
  const row = await db
    .select({ expiresAt: sessions.expiresAt })
    .from(sessions)
    .where(eq(sessions.id, sessionId))
    .get();
  return row?.expiresAt ?? null;
}
