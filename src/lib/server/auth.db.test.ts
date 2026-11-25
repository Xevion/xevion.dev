import { beforeEach, describe, expect, it } from "vitest";
import { resetDb, testDb } from "./db/test-db.ts";
import { changePassword, checkCredentials, createUser } from "./auth.ts";
import { createSession, resolveSession } from "./sessions.ts";

const db = testDb();
let userId: number;

beforeEach(async () => {
  await resetDb();
  userId = await createUser(db, "admin", "correct horse battery");
});

describe("changePassword", () => {
  it("rejects a wrong current password and keeps the old one", async () => {
    const current = await createSession(db, userId, "browser");
    const result = await changePassword(
      db,
      userId,
      current.id,
      "wrong",
      "a brand new passphrase",
    );
    expect(result).toEqual({ changed: false });
    expect(
      await checkCredentials(db, "admin", "correct horse battery"),
    ).not.toBeNull();
  });

  it("swaps the password and signs out every other browser session", async () => {
    const current = await createSession(db, userId, "browser");
    const other = await createSession(db, userId, "browser");
    const cli = await createSession(db, userId, "cli", "laptop");

    const result = await changePassword(
      db,
      userId,
      current.id,
      "correct horse battery",
      "a brand new passphrase",
    );

    expect(result).toEqual({ changed: true, revokedSessions: 1 });
    expect(await checkCredentials(db, "admin", "correct horse battery")).toBe(
      null,
    );
    expect(
      await checkCredentials(db, "admin", "a brand new passphrase"),
    ).not.toBeNull();
    expect(await resolveSession(db, current.token)).not.toBeNull();
    expect(await resolveSession(db, other.token)).toBeNull();
    // Device-approved CLI tokens never held the password, so they survive.
    expect(await resolveSession(db, cli.token)).not.toBeNull();
  });
});
