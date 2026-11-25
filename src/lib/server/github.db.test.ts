import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { env } from "cloudflare:workers";
import { eq } from "drizzle-orm";
import { resetDb, testDb } from "./db/test-db.ts";
import { appState, projects } from "./db/schema.ts";
import {
  PAUSED_UNTIL_KEY,
  enqueueAllGithubSyncs,
  getGithubSyncStatus,
  runGithubSync,
} from "./github.ts";

const db = testDb();
const ACTIVITY = "2026-05-01T12:00:00Z";
const ISSUE = "2026-05-02T08:00:00Z";

interface Route {
  status?: number;
  body?: unknown;
  headers?: Record<string, string>;
}

function stubGithub(routes: Record<string, Route>) {
  const calls: { url: string; headers: Headers }[] = [];
  vi.stubGlobal(
    "fetch",
    vi.fn(async (url: string, init?: RequestInit) => {
      const headers = new Headers(init?.headers);
      calls.push({ url, headers });
      const key = Object.keys(routes).find((k) => url.includes(k));
      const route = key ? routes[key]! : { status: 500 };
      return new Response(
        route.status === 304 ? null : JSON.stringify(route.body ?? []),
        { status: route.status ?? 200, headers: route.headers },
      );
    }),
  );
  return calls;
}

async function addProject(values: Partial<typeof projects.$inferInsert> = {}) {
  const [row] = await db
    .insert(projects)
    .values({
      slug: "p",
      name: "P",
      githubRepo: "owner/repo",
      updatedAt: new Date("2026-01-01T00:00:00Z"),
      ...values,
    })
    .returning();
  return row!;
}

const load = (id: string) =>
  db.select().from(projects).where(eq(projects.id, id)).get();

const healthy: Record<string, Route> = {
  "/activity": { body: [{ timestamp: ACTIVITY }] },
  "/issues": { body: [{ updated_at: ISSUE }], headers: { etag: 'W/"v1"' } },
  "/repos/owner/repo": { body: { id: 42, full_name: "owner/repo" } },
};

beforeEach(async () => {
  await resetDb();
  (env as { GITHUB_TOKEN: string }).GITHUB_TOKEN = "test-token";
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("runGithubSync", () => {
  it("records the newest signal, the etag and the schedule without touching updated_at", async () => {
    const calls = stubGithub(healthy);
    const project = await addProject();

    const summary = await runGithubSync(db);
    expect(summary).toMatchObject({ synced: 1, errors: 0, skipped: null });
    expect(calls).toHaveLength(3);
    expect(calls[0]!.headers.get("authorization")).toBe("Bearer test-token");
    expect(calls[0]!.headers.get("user-agent")).toBe("xevion-dev/1.0");

    const row = (await load(project.id))!;
    expect(row.lastGithubActivity?.toISOString()).toBe(
      "2026-05-02T08:00:00.000Z",
    );
    expect(row.githubEtag).toBe('W/"v1"');
    expect(row.githubEtagValue?.toISOString()).toBe("2026-05-02T08:00:00.000Z");
    expect(row.githubRepoId).toBe(42);
    expect(row.githubSyncError).toBeNull();
    expect(row.githubErrorCount).toBe(0);
    expect(row.githubNextSyncAt!.getTime()).toBeGreaterThan(Date.now());
    expect(row.updatedAt.toISOString()).toBe("2026-01-01T00:00:00.000Z");
  });

  it("reuses the stored etag value on a 304 and never regresses activity", async () => {
    const calls = stubGithub({
      ...healthy,
      "/activity": { body: [] },
      "/issues": { status: 304 },
    });
    const project = await addProject({
      githubEtag: 'W/"v1"',
      githubEtagValue: new Date(ISSUE),
      lastGithubActivity: new Date("2026-06-01T00:00:00Z"),
    });

    await runGithubSync(db);

    expect(
      calls
        .find((c) => c.url.includes("/issues"))!
        .headers.get("if-none-match"),
    ).toBe('W/"v1"');
    const row = (await load(project.id))!;
    expect(row.lastGithubActivity?.toISOString()).toBe(
      "2026-06-01T00:00:00.000Z",
    );
    expect(row.githubEtag).toBe('W/"v1"');
  });

  it("heals a renamed repo and only writes identity when it differs", async () => {
    stubGithub({
      ...healthy,
      "/repos/owner/repo": { body: { id: 7, full_name: "owner/renamed" } },
    });
    const project = await addProject();

    await runGithubSync(db);

    const row = (await load(project.id))!;
    expect(row.githubRepo).toBe("owner/renamed");
    expect(row.githubRepoId).toBe(7);
    expect(row.updatedAt.toISOString()).toBe("2026-01-01T00:00:00.000Z");
  });

  it("backs off exponentially on errors and recovers on success", async () => {
    stubGithub({ "/activity": { status: 500 }, "/issues": { status: 500 } });
    const project = await addProject({ lastGithubActivity: new Date() });

    await runGithubSync(db);
    let row = (await load(project.id))!;
    expect(row.githubErrorCount).toBe(1);
    expect(row.githubSyncError).toContain("500");
    const firstWait = row.githubNextSyncAt!.getTime() - Date.now();

    await db
      .update(projects)
      .set({ githubNextSyncAt: new Date(Date.now() - 1000) });
    await runGithubSync(db);
    row = (await load(project.id))!;
    expect(row.githubErrorCount).toBe(2);
    expect(row.githubNextSyncAt!.getTime() - Date.now()).toBeGreaterThan(
      firstWait * 1.5,
    );

    stubGithub(healthy);
    await db
      .update(projects)
      .set({ githubNextSyncAt: new Date(Date.now() - 1000) });
    await runGithubSync(db);
    row = (await load(project.id))!;
    expect(row.githubErrorCount).toBe(0);
    expect(row.githubSyncError).toBeNull();
  });

  it("records a missing repo without escalating the backoff", async () => {
    stubGithub({ "/repos/owner/repo": { status: 404 } });
    const project = await addProject();

    await runGithubSync(db);

    const row = (await load(project.id))!;
    expect(row.githubSyncError).toBe("Repository not found or inaccessible");
    expect(row.githubErrorCount).toBe(0);
    expect(row.githubNextSyncAt).not.toBeNull();
  });

  it("pauses every run after a rate limit and keeps the project's last real error", async () => {
    const calls = stubGithub({
      "/repos/owner/repo": {
        status: 403,
        headers: { "x-ratelimit-remaining": "0", "retry-after": "600" },
      },
    });
    const first = await addProject({ githubSyncError: "old" });
    const second = await addProject({ slug: "q", githubRepo: "owner/other" });

    const summary = await runGithubSync(db);
    expect(summary.rateLimitedUntil).not.toBeNull();
    // The first project's three calls all fail; the second is never attempted.
    expect(calls).toHaveLength(3);
    expect((await load(first.id))!.githubSyncError).toBe("old");
    expect((await load(first.id))!.githubErrorCount).toBe(1);
    expect((await load(second.id))!.githubNextSyncAt).toBeNull();

    const paused = await db
      .select()
      .from(appState)
      .where(eq(appState.key, PAUSED_UNTIL_KEY))
      .get();
    expect(Number(paused!.value) - Date.now()).toBeGreaterThan(590_000);

    expect(await runGithubSync(db)).toMatchObject({
      synced: 0,
      skipped: "paused",
    });
    expect(calls).toHaveLength(3);
    expect((await getGithubSyncStatus(db)).pausedUntil).not.toBeNull();
  });

  it("does nothing without a token", async () => {
    (env as { GITHUB_TOKEN: string }).GITHUB_TOKEN = "";
    const calls = stubGithub(healthy);
    await addProject();

    expect(await runGithubSync(db)).toMatchObject({ skipped: "no-token" });
    expect(calls).toHaveLength(0);
  });

  it("picks only due projects, never-synced first, within the limit", async () => {
    stubGithub(healthy);
    const later = await addProject({
      slug: "later",
      githubNextSyncAt: new Date(Date.now() + 3600_000),
    });
    const overdue = await addProject({
      slug: "overdue",
      githubNextSyncAt: new Date(Date.now() - 3600_000),
    });
    const never = await addProject({ slug: "never" });
    await addProject({ slug: "no-repo", githubRepo: null });

    expect(await runGithubSync(db, 1)).toMatchObject({ synced: 1 });
    expect((await load(never.id))!.githubSyncedAt).not.toBeNull();
    expect((await load(overdue.id))!.githubSyncedAt).toBeNull();

    await runGithubSync(db);
    expect((await load(overdue.id))!.githubSyncedAt).not.toBeNull();
    expect((await load(later.id))!.githubSyncedAt).toBeNull();
  });
});

describe("sync status", () => {
  it("counts tracked projects and re-enqueues them all", async () => {
    await addProject({ slug: "a", githubSyncError: "boom" });
    await addProject({
      slug: "b",
      githubSyncedAt: new Date(),
      githubNextSyncAt: new Date(Date.now() + 3600_000),
    });
    await addProject({ slug: "c", githubRepo: null });

    expect(await getGithubSyncStatus(db)).toMatchObject({
      tracked: 2,
      neverSynced: 1,
      erroring: 1,
      due: 1,
      pausedUntil: null,
    });

    expect(await enqueueAllGithubSyncs(db)).toBe(2);
    expect((await getGithubSyncStatus(db)).due).toBe(2);
  });
});
