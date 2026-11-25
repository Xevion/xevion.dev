import { and, eq, ne, sql } from "drizzle-orm";
import type { Db } from "./db/index.ts";
import { sessions, users } from "./db/schema.ts";
import { hashPassword, verifyPassword } from "./crypto.ts";

export const MIN_PASSWORD_LENGTH = 12;
export const MAX_PASSWORD_LENGTH = 256;

export async function hasAnyUser(db: Db): Promise<boolean> {
  const row = await db
    .select({ n: sql<number>`count(*)` })
    .from(users)
    .get();
  return (row?.n ?? 0) > 0;
}

export async function createUser(
  db: Db,
  username: string,
  password: string,
): Promise<number> {
  const [row] = await db
    .insert(users)
    .values({ username, passwordHash: await hashPassword(password) })
    .returning({ id: users.id });
  return row!.id;
}

/** Returns the user id when the credentials match. */
export async function checkCredentials(
  db: Db,
  username: string,
  password: string,
): Promise<{ id: number; username: string } | null> {
  const row = await db
    .select()
    .from(users)
    .where(eq(users.username, username))
    .get();
  if (!row) {
    // Spend the same derivation time as a real check so a missing user isn't distinguishable.
    await verifyPassword(password, DUMMY_HASH);
    return null;
  }
  return (await verifyPassword(password, row.passwordHash))
    ? { id: row.id, username: row.username }
    : null;
}

export async function setPassword(
  db: Db,
  userId: number,
  password: string,
): Promise<void> {
  await db
    .update(users)
    .set({ passwordHash: await hashPassword(password) })
    .where(eq(users.id, userId));
}

export type PasswordChange =
  | { changed: false }
  | { changed: true; revokedSessions: number };

/**
 * Replaces the password after checking the current one, and signs out every other browser session.
 * CLI tokens stay: they were approved through the device flow and never held the password.
 */
export async function changePassword(
  db: Db,
  userId: number,
  currentSessionId: string,
  currentPassword: string,
  newPassword: string,
): Promise<PasswordChange> {
  const row = await db.select().from(users).where(eq(users.id, userId)).get();
  if (!row || !(await verifyPassword(currentPassword, row.passwordHash))) {
    return { changed: false };
  }
  const passwordHash = await hashPassword(newPassword);
  const [, revoked] = await db.batch([
    db.update(users).set({ passwordHash }).where(eq(users.id, userId)),
    db
      .delete(sessions)
      .where(
        and(
          eq(sessions.userId, userId),
          eq(sessions.kind, "browser"),
          ne(sessions.id, currentSessionId),
        ),
      )
      .returning({ id: sessions.id }),
  ]);
  return { changed: true, revokedSessions: revoked.length };
}

const DUMMY_HASH =
  "pbkdf2-sha256$100000$AAAAAAAAAAAAAAAAAAAAAA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
