import { and, eq, lt, ne } from "drizzle-orm";
import type { Db } from "./db/index.ts";
import { deviceAuthRequests } from "./db/schema.ts";
import { randomToken } from "./crypto.ts";
import { createSession, revokeSession } from "./sessions.ts";

// Kept free of $app/* and cloudflare:workers so workerd db tests can import it directly.

export const DEVICE_REQUEST_TTL_MS = 10 * 60_000;
export const DEVICE_POLL_INTERVAL_SECONDS = 2;

// No 0/O/1/I so a code read off a terminal can't be mistyped; 32 symbols divide 256 evenly.
const USER_CODE_ALPHABET = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const USER_CODE_LENGTH = 8;
const MAX_LABEL_LENGTH = 64;

export function generateUserCode(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(USER_CODE_LENGTH));
  const chars = Array.from(
    bytes,
    (b) => USER_CODE_ALPHABET[b % USER_CODE_ALPHABET.length],
  ).join("");
  return `${chars.slice(0, 4)}-${chars.slice(4)}`;
}

export function normalizeUserCode(input: string): string {
  return input.replace(/[^A-Za-z0-9]/g, "").toUpperCase();
}

export interface StartedDeviceAuth {
  id: string;
  userCode: string;
  expiresAt: Date;
  interval: number;
}

export async function startDeviceAuth(
  db: Db,
  label?: string,
): Promise<StartedDeviceAuth> {
  const id = randomToken(32);
  const userCode = generateUserCode();
  const now = new Date();
  await db.insert(deviceAuthRequests).values({
    id,
    userCode,
    label: label?.trim().slice(0, MAX_LABEL_LENGTH) || null,
    createdAt: now,
  });
  return {
    id,
    userCode,
    expiresAt: new Date(now.getTime() + DEVICE_REQUEST_TTL_MS),
    interval: DEVICE_POLL_INTERVAL_SECONDS,
  };
}

type RequestRow = typeof deviceAuthRequests.$inferSelect;

const expiresAtOf = (row: RequestRow) =>
  new Date(row.createdAt.getTime() + DEVICE_REQUEST_TTL_MS);

export type DevicePeek =
  | { status: "pending"; label: string | null; expiresAt: Date }
  | { status: "approved" | "denied" }
  | { status: "expired" };

/** Read-only view for the approval page; never consumes the request. */
export async function peekDeviceAuth(db: Db, id: string): Promise<DevicePeek> {
  const row = await db
    .select()
    .from(deviceAuthRequests)
    .where(eq(deviceAuthRequests.id, id))
    .get();
  if (!row) return { status: "expired" };
  if (row.status !== "pending") return { status: row.status };
  if (expiresAtOf(row).getTime() <= Date.now()) return { status: "expired" };
  return { status: "pending", label: row.label, expiresAt: expiresAtOf(row) };
}

export type ApproveResult = "approved" | "mismatch" | "expired" | "not-pending";

/**
 * Marks a pending request approved after the approver retyped the code shown by the CLI, minting
 * the CLI session whose plaintext token waits on the row until the poll collects it.
 */
export async function approveDeviceAuth(
  db: Db,
  id: string,
  userId: number,
  typedCode: string,
): Promise<ApproveResult> {
  const row = await db
    .select()
    .from(deviceAuthRequests)
    .where(eq(deviceAuthRequests.id, id))
    .get();
  if (!row) return "expired";
  if (row.status !== "pending") return "not-pending";
  if (expiresAtOf(row).getTime() <= Date.now()) return "expired";
  if (normalizeUserCode(typedCode) !== normalizeUserCode(row.userCode))
    return "mismatch";

  const session = await createSession(
    db,
    userId,
    "cli",
    row.label ?? undefined,
  );
  const updated = await db
    .update(deviceAuthRequests)
    .set({ status: "approved", token: session.token })
    .where(
      and(
        eq(deviceAuthRequests.id, id),
        eq(deviceAuthRequests.status, "pending"),
      ),
    )
    .returning({ id: deviceAuthRequests.id });
  if (updated.length === 0) {
    await revokeSession(db, userId, session.id);
    return "not-pending";
  }
  return "approved";
}

export async function denyDeviceAuth(db: Db, id: string): Promise<boolean> {
  const updated = await db
    .update(deviceAuthRequests)
    .set({ status: "denied" })
    .where(
      and(
        eq(deviceAuthRequests.id, id),
        eq(deviceAuthRequests.status, "pending"),
      ),
    )
    .returning({ id: deviceAuthRequests.id });
  return updated.length > 0;
}

export type PollResult =
  | { status: "pending" }
  | { status: "approved"; token: string }
  | { status: "denied" }
  | { status: "unknown" };

/** Reports the request's state; an approved or denied outcome is delivered once and the row removed. */
export async function pollDeviceAuth(db: Db, id: string): Promise<PollResult> {
  const row = await db
    .select()
    .from(deviceAuthRequests)
    .where(eq(deviceAuthRequests.id, id))
    .get();
  if (!row) return { status: "unknown" };

  if (row.status === "pending") {
    if (expiresAtOf(row).getTime() > Date.now()) return { status: "pending" };
    await db
      .delete(deviceAuthRequests)
      .where(
        and(
          eq(deviceAuthRequests.id, id),
          eq(deviceAuthRequests.status, "pending"),
        ),
      );
    return { status: "unknown" };
  }

  // Deleting with the status guard makes concurrent polls race for one delivery.
  const [taken] = await db
    .delete(deviceAuthRequests)
    .where(
      and(
        eq(deviceAuthRequests.id, id),
        ne(deviceAuthRequests.status, "pending"),
      ),
    )
    .returning();
  if (!taken) return { status: "unknown" };
  if (taken.status === "approved" && taken.token)
    return { status: "approved", token: taken.token };
  return { status: "denied" };
}

export async function deleteStaleDeviceRequests(db: Db): Promise<number> {
  const deleted = await db
    .delete(deviceAuthRequests)
    .where(
      lt(
        deviceAuthRequests.createdAt,
        new Date(Date.now() - DEVICE_REQUEST_TTL_MS),
      ),
    )
    .returning({ id: deviceAuthRequests.id });
  return deleted.length;
}
