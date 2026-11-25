import { beforeEach, describe, expect, it } from "vitest";
import { eq } from "drizzle-orm";
import { resetDb, testDb } from "./db/test-db.ts";
import { deviceAuthRequests, sessions } from "./db/schema.ts";
import { createUser } from "./auth.ts";
import {
  DEVICE_REQUEST_TTL_MS,
  approveDeviceAuth,
  deleteStaleDeviceRequests,
  denyDeviceAuth,
  peekDeviceAuth,
  pollDeviceAuth,
  startDeviceAuth,
} from "./device-auth.ts";
import {
  CLI_TOKEN_PREFIX,
  CLI_SESSION_TTL_MS,
  createSession,
  resolveSession,
} from "./sessions.ts";

const db = testDb();
let userId: number;

beforeEach(async () => {
  await resetDb();
  userId = await createUser(db, "admin", "correct horse battery");
});

const age = (id: string, ms: number) =>
  db
    .update(deviceAuthRequests)
    .set({ createdAt: new Date(Date.now() - ms) })
    .where(eq(deviceAuthRequests.id, id));

describe("device auth", () => {
  it("issues an unambiguous XXXX-XXXX code and a secret id", async () => {
    const started = await startDeviceAuth(db, "laptop");
    expect(started.userCode).toMatch(/^[A-HJ-NP-Z2-9]{4}-[A-HJ-NP-Z2-9]{4}$/);
    expect(started.id.length).toBeGreaterThanOrEqual(43);
    expect(started.interval).toBe(2);
    expect(await pollDeviceAuth(db, started.id)).toEqual({ status: "pending" });
  });

  it("refuses a wrong code, then approves a retyped one case-insensitively", async () => {
    const { id, userCode } = await startDeviceAuth(db, "laptop");

    expect(await approveDeviceAuth(db, id, userId, "AAAA-AAAA")).toBe(
      "mismatch",
    );
    expect(await pollDeviceAuth(db, id)).toEqual({ status: "pending" });
    expect(await db.select().from(sessions)).toHaveLength(0);

    const retyped = userCode.toLowerCase().replace("-", " ");
    expect(await approveDeviceAuth(db, id, userId, retyped)).toBe("approved");
  });

  it("delivers the token exactly once and the token authenticates as a CLI session", async () => {
    const { id, userCode } = await startDeviceAuth(db, "laptop");
    await approveDeviceAuth(db, id, userId, userCode);

    const first = await pollDeviceAuth(db, id);
    assertApproved(first);
    expect(first.token.startsWith(CLI_TOKEN_PREFIX)).toBe(true);
    expect(await pollDeviceAuth(db, id)).toEqual({ status: "unknown" });

    const resolved = await resolveSession(db, first.token);
    expect(resolved?.user).toMatchObject({
      id: userId,
      sessionKind: "cli",
    });
    const [row] = await db.select().from(sessions);
    expect(row?.label).toBe("laptop");
  });

  it("does not consume the request when peeked", async () => {
    const { id, userCode } = await startDeviceAuth(db);
    await approveDeviceAuth(db, id, userId, userCode);
    expect(await peekDeviceAuth(db, id)).toEqual({ status: "approved" });
    expect(await peekDeviceAuth(db, id)).toEqual({ status: "approved" });
    assertApproved(await pollDeviceAuth(db, id));
  });

  it("delivers a denial once", async () => {
    const { id } = await startDeviceAuth(db);
    expect(await denyDeviceAuth(db, id)).toBe(true);
    expect(await denyDeviceAuth(db, id)).toBe(false);
    expect(await pollDeviceAuth(db, id)).toEqual({ status: "denied" });
    expect(await pollDeviceAuth(db, id)).toEqual({ status: "unknown" });
  });

  it("rejects approval of an already resolved request", async () => {
    const { id, userCode } = await startDeviceAuth(db);
    await approveDeviceAuth(db, id, userId, userCode);
    expect(await approveDeviceAuth(db, id, userId, userCode)).toBe(
      "not-pending",
    );
    expect(await db.select().from(sessions)).toHaveLength(1);
  });

  it("treats a pending request past its ttl as expired everywhere", async () => {
    const { id, userCode } = await startDeviceAuth(db);
    await age(id, DEVICE_REQUEST_TTL_MS + 1000);

    expect(await peekDeviceAuth(db, id)).toEqual({ status: "expired" });
    expect(await approveDeviceAuth(db, id, userId, userCode)).toBe("expired");
    expect(await pollDeviceAuth(db, id)).toEqual({ status: "unknown" });
    expect(await peekDeviceAuth(db, "no-such-id")).toEqual({
      status: "expired",
    });
  });

  it("sweeps only requests older than the ttl", async () => {
    const stale = await startDeviceAuth(db);
    const fresh = await startDeviceAuth(db);
    await age(stale.id, DEVICE_REQUEST_TTL_MS + 1000);

    expect(await deleteStaleDeviceRequests(db)).toBe(1);
    expect(await pollDeviceAuth(db, fresh.id)).toEqual({ status: "pending" });
  });
});

describe("resolveSession lifetime", () => {
  it("drops an expired session and its row", async () => {
    const { token, id } = await createSession(db, userId, "browser");
    await db
      .update(sessions)
      .set({ expiresAt: new Date(Date.now() - 1000) })
      .where(eq(sessions.id, id));

    expect(await resolveSession(db, token)).toBeNull();
    expect(await db.select().from(sessions)).toHaveLength(0);
  });

  it("slides a CLI session's expiry but not a browser session's", async () => {
    const cli = await createSession(db, userId, "cli");
    const browser = await createSession(db, userId, "browser");
    const nearEnd = new Date(Date.now() + 24 * 3600_000);
    await db.update(sessions).set({ expiresAt: nearEnd });

    const cliResolved = await resolveSession(db, cli.token);
    const browserResolved = await resolveSession(db, browser.token);
    await cliResolved?.touch?.();
    await browserResolved?.touch?.();

    const rows = await db.select().from(sessions);
    const expiry = (id: string) =>
      rows.find((r) => r.id === id)!.expiresAt.getTime();
    expect(expiry(cli.id)).toBeGreaterThan(
      Date.now() + CLI_SESSION_TTL_MS - 60_000,
    );
    expect(expiry(browser.id)).toBe(
      Math.floor(nearEnd.getTime() / 1000) * 1000,
    );
  });
});

function assertApproved(
  result: Awaited<ReturnType<typeof pollDeviceAuth>>,
): asserts result is { status: "approved"; token: string } {
  expect(result.status).toBe("approved");
}
