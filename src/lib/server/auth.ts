import { eq, sql } from "drizzle-orm";
import type { Db } from "./db/index.ts";
import { users } from "./db/schema.ts";
import { hashPassword, verifyPassword } from "./crypto.ts";

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

const DUMMY_HASH =
  "pbkdf2-sha256$100000$AAAAAAAAAAAAAAAAAAAAAA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
