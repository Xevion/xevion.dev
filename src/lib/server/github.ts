import { and, asc, eq, isNotNull, isNull, lte, ne, or, sql } from "drizzle-orm";
import type { Db } from "./db/index.ts";
import { appState, projects } from "./db/schema.ts";
import { getLogger } from "@logtape/logtape";

// Kept free of static cloudflare:workers and $app/* imports (including the app logger) so the pure
// parts run under plain node.

const logger = getLogger(["xevion", "github"]);

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;

export const MIN_INTERVAL_MS = 15 * MINUTE;
export const MAX_INTERVAL_MS = 24 * HOUR;
const DAYS_TO_MAX = 90;
const RATE_LIMIT_FALLBACK_MS = 5 * MINUTE;
const MAX_BACKOFF_EXPONENT = 6;

export const PAUSED_UNTIL_KEY = "github_paused_until";

const API = "https://api.github.com";

/** Accepts `owner/repo` or a pasted GitHub URL; returns `owner/repo` or null when malformed. */
export function normalizeGithubRepo(input: string): string | null {
  let stripped = input.trim();
  for (const prefix of [
    "https://github.com/",
    "http://github.com/",
    "github.com/",
  ]) {
    if (stripped.startsWith(prefix)) {
      stripped = stripped.slice(prefix.length);
      break;
    }
  }
  if (stripped.endsWith("/")) stripped = stripped.slice(0, -1);
  if (stripped.endsWith(".git")) stripped = stripped.slice(0, -".git".length);

  const parts = stripped.split("/");
  if (parts.length !== 2) return null;
  const [owner, repo] = parts as [string, string];
  if (
    !owner ||
    !repo ||
    owner.length > 39 ||
    !/^[A-Za-z0-9-]+$/.test(owner) ||
    !/^[A-Za-z0-9._-]+$/.test(repo)
  )
    return null;
  return `${owner}/${repo}`;
}

/** Splits a canonical `owner/repo`; null when it is not exactly two non-empty segments. */
export function parseGithubRepo(
  githubRepo: string,
): { owner: string; repo: string } | null {
  const [owner, repo, ...rest] = githubRepo.split("/");
  if (!owner || !repo || rest.length > 0) return null;
  return { owner, repo };
}

/** Log-scaled wait before the next check: 15 min for fresh activity up to 24 h at 90+ idle days. */
export function calculateCheckIntervalMs(
  lastActivity: Date | null,
  now: number = Date.now(),
): number {
  const daysSince = lastActivity
    ? Math.max(0, Math.floor((now - lastActivity.getTime()) / (24 * HOUR)))
    : DAYS_TO_MAX;
  const scale = Math.min(
    1,
    Math.max(0, Math.log1p(daysSince) / Math.log1p(DAYS_TO_MAX)),
  );
  const seconds =
    MIN_INTERVAL_MS / 1000 +
    scale * ((MAX_INTERVAL_MS - MIN_INTERVAL_MS) / 1000);
  return Math.floor(seconds) * 1000;
}

/** Interval after the `errorCount`-th consecutive failure: doubles each time, capped at 24 h. */
export function errorBackoffMs(
  lastActivity: Date | null,
  errorCount: number,
  now: number = Date.now(),
): number {
  const base = calculateCheckIntervalMs(lastActivity, now);
  const multiplier =
    2 ** Math.min(Math.max(errorCount, 0), MAX_BACKOFF_EXPONENT);
  return Math.min(base * multiplier, MAX_INTERVAL_MS);
}

/** Later of two timestamps, treating null as no signal. */
export function mergeActivity(a: Date | null, b: Date | null): Date | null {
  if (a && b) return a.getTime() >= b.getTime() ? a : b;
  return a ?? b;
}

/** GitHub signals rate limiting with 429, or 403 once the primary quota is spent. */
export function isRateLimited(
  status: number,
  remaining: string | null,
): boolean {
  return status === 429 || (status === 403 && remaining === "0");
}

/** Wait from `retry-after` seconds, else `x-ratelimit-reset` (unix seconds); null when neither. */
export function rateLimitBackoffMs(
  retryAfter: string | null,
  resetAt: string | null,
  nowUnixSeconds: number,
): number | null {
  const retry = retryAfter?.trim();
  if (retry && /^\d+$/.test(retry)) return Number(retry) * 1000;
  const reset = resetAt?.trim();
  if (reset && /^-?\d+$/.test(reset))
    return Math.max(0, Number(reset) - nowUnixSeconds) * 1000;
  return null;
}

class GithubError extends Error {
  constructor(
    readonly kind: "not-found" | "rate-limited" | "api",
    message: string,
    readonly retryAfterMs: number | null = null,
    readonly status = 0,
  ) {
    super(message);
  }
}

async function readToken(): Promise<string | undefined> {
  const { env } = await import("cloudflare:workers");
  return env.GITHUB_TOKEN || undefined;
}

function headers(token: string, extra?: Record<string, string>): HeadersInit {
  return {
    accept: "application/vnd.github+json",
    "x-github-api-version": "2022-11-28",
    authorization: `Bearer ${token}`,
    "user-agent": "xevion-dev/1.0",
    ...extra,
  };
}

async function ensureOk(response: Response, slug: string): Promise<Response> {
  if (response.ok) return response;
  if (response.status === 404)
    throw new GithubError(
      "not-found",
      `Repository not found or inaccessible: ${slug}`,
    );
  if (
    isRateLimited(
      response.status,
      response.headers.get("x-ratelimit-remaining"),
    )
  ) {
    throw new GithubError(
      "rate-limited",
      "GitHub API rate limit exceeded",
      rateLimitBackoffMs(
        response.headers.get("retry-after"),
        response.headers.get("x-ratelimit-reset"),
        Math.floor(Date.now() / 1000),
      ),
      response.status,
    );
  }
  const body = await response.text().catch(() => "");
  throw new GithubError(
    "api",
    `GitHub API error (${response.status}): ${body}`,
    null,
    response.status,
  );
}

function parseTimestamp(value: string): Date {
  const date = new Date(value);
  if (Number.isNaN(date.getTime()))
    throw new GithubError("api", `Failed to parse timestamp: ${value}`);
  return date;
}

async function getJson<T>(
  url: string,
  token: string,
  slug: string,
): Promise<T> {
  const response = await ensureOk(
    await fetch(url, { headers: headers(token) }),
    slug,
  );
  return (await response.json()) as T;
}

async function fetchActivityTimestamp(
  token: string,
  slug: string,
): Promise<Date | null> {
  const items = await getJson<{ timestamp: string }[]>(
    `${API}/repos/${slug}/activity?per_page=1`,
    token,
    slug,
  );
  return items[0] ? parseTimestamp(items[0].timestamp) : null;
}

interface IssueCache {
  etag: string | null;
  timestamp: Date | null;
}

async function fetchIssueTimestamp(
  token: string,
  slug: string,
  cached: IssueCache,
): Promise<IssueCache> {
  const response = await fetch(
    `${API}/repos/${slug}/issues?state=all&sort=updated&direction=desc&per_page=1`,
    {
      headers: headers(
        token,
        cached.etag ? { "if-none-match": cached.etag } : undefined,
      ),
    },
  );
  if (response.status === 304) return cached;

  const etag = response.headers.get("etag");
  await ensureOk(response, slug);
  const items = (await response.json()) as { updated_at: string }[];
  return {
    etag,
    timestamp: items[0] ? parseTimestamp(items[0].updated_at) : null,
  };
}

async function fetchRepoMeta(
  token: string,
  slug: string,
): Promise<{ id: number; fullName: string }> {
  const body = await getJson<{ id: number; full_name: string }>(
    `${API}/repos/${slug}`,
    token,
    slug,
  );
  return { id: body.id, fullName: body.full_name };
}

export type RepoLookup =
  | { kind: "found"; id: number; fullName: string }
  | { kind: "not-found" }
  | { kind: "error"; message: string };

/** GET /repos/{owner}/{repo}, following renames. */
export async function fetchRepo(repo: string): Promise<RepoLookup> {
  const parsed = parseGithubRepo(repo);
  if (!parsed)
    return { kind: "error", message: `Invalid github_repo: ${repo}` };
  const token = await readToken();
  if (!token)
    return { kind: "error", message: "GitHub sync disabled (no token)" };
  try {
    const meta = await fetchRepoMeta(token, repo);
    return { kind: "found", ...meta };
  } catch (cause) {
    if (cause instanceof GithubError && cause.kind === "not-found")
      return { kind: "not-found" };
    return {
      kind: "error",
      message: cause instanceof Error ? cause.message : String(cause),
    };
  }
}

type ProjectRow = typeof projects.$inferSelect;
type SyncTarget = Pick<
  ProjectRow,
  | "id"
  | "githubRepo"
  | "githubRepoId"
  | "lastGithubActivity"
  | "githubEtag"
  | "githubEtagValue"
  | "githubErrorCount"
>;

type SyncOutcome =
  | { kind: "ok"; changed: boolean; renamedTo: string | null }
  | { kind: "not-found" }
  | { kind: "invalid-repo" }
  | { kind: "rate-limited"; retryAfterMs: number | null }
  | { kind: "error"; message: string };

const toSeconds = (date: Date) => Math.floor(date.getTime() / 1000);

async function recordFailure(
  db: Db,
  project: SyncTarget,
  message: string | null,
  nextDelayMs: number,
  errorCount: number,
): Promise<void> {
  await db
    .update(projects)
    .set({
      // A rate limit is not the project's fault, so it keeps showing its last real error.
      ...(message === null ? {} : { githubSyncError: message }),
      githubErrorCount: errorCount,
      githubNextSyncAt: new Date(Date.now() + nextDelayMs),
    })
    .where(eq(projects.id, project.id));
}

async function syncProject(
  db: Db,
  token: string,
  project: SyncTarget,
): Promise<SyncOutcome> {
  const slug = project.githubRepo;
  const parsed = slug ? parseGithubRepo(slug) : null;
  if (!slug || !parsed) {
    await recordFailure(
      db,
      project,
      "Invalid github_repo format",
      calculateCheckIntervalMs(project.lastGithubActivity),
      0,
    );
    return { kind: "invalid-repo" };
  }

  const cached: IssueCache = {
    etag: project.githubEtag,
    timestamp: project.githubEtagValue,
  };
  const [activity, issues, meta] = await Promise.allSettled([
    fetchActivityTimestamp(token, slug),
    fetchIssueTimestamp(token, slug, cached),
    fetchRepoMeta(token, slug),
  ]);

  // Activity and issues drive retry/backoff; identity is best-effort healing.
  const failure = [activity, issues].find((r) => r.status === "rejected");
  if (failure) {
    const cause = failure.reason as unknown;
    const message = cause instanceof Error ? cause.message : String(cause);
    if (cause instanceof GithubError && cause.kind === "rate-limited") {
      const errorCount = project.githubErrorCount + 1;
      await recordFailure(
        db,
        project,
        null,
        errorBackoffMs(project.lastGithubActivity, errorCount),
        errorCount,
      );
      return { kind: "rate-limited", retryAfterMs: cause.retryAfterMs };
    }
    if (cause instanceof GithubError && cause.kind === "not-found") {
      await recordFailure(
        db,
        project,
        "Repository not found or inaccessible",
        calculateCheckIntervalMs(project.lastGithubActivity),
        0,
      );
      return { kind: "not-found" };
    }
    const errorCount = project.githubErrorCount + 1;
    await recordFailure(
      db,
      project,
      message,
      errorBackoffMs(project.lastGithubActivity, errorCount),
      errorCount,
    );
    return { kind: "error", message };
  }

  const latest = mergeActivity(
    (activity as PromiseFulfilledResult<Date | null>).value,
    (issues as PromiseFulfilledResult<IssueCache>).value.timestamp,
  );
  const issueState = (issues as PromiseFulfilledResult<IssueCache>).value;
  const effective = mergeActivity(project.lastGithubActivity, latest);

  await db
    .update(projects)
    .set({
      ...(latest
        ? {
            lastGithubActivity: sql`coalesce(max(${projects.lastGithubActivity}, ${toSeconds(latest)}), ${toSeconds(latest)})`,
          }
        : {}),
      githubSyncedAt: new Date(),
      githubSyncError: null,
      githubErrorCount: 0,
      githubEtag: issueState.etag,
      githubEtagValue: issueState.etag ? issueState.timestamp : null,
      githubNextSyncAt: new Date(
        Date.now() + calculateCheckIntervalMs(effective),
      ),
    })
    .where(eq(projects.id, project.id));

  let renamedTo: string | null = null;
  if (meta.status === "fulfilled") {
    const { id, fullName } = meta.value;
    try {
      await db
        .update(projects)
        .set({ githubRepo: fullName, githubRepoId: id })
        .where(
          and(
            eq(projects.id, project.id),
            or(
              isNull(projects.githubRepo),
              ne(projects.githubRepo, fullName),
              isNull(projects.githubRepoId),
              ne(projects.githubRepoId, id),
            ),
          ),
        );
      if (fullName !== slug) renamedTo = fullName;
    } catch (cause) {
      logger.warn("failed to persist github repo identity", {
        projectId: project.id,
        cause: String(cause),
      });
    }
  } else {
    logger.debug("repo identity resolve failed", {
      repo: slug,
      cause: String(meta.reason),
    });
  }

  const changed = latest
    ? !project.lastGithubActivity ||
      latest.getTime() > project.lastGithubActivity.getTime()
    : false;
  return { kind: "ok", changed, renamedTo };
}

async function getPausedUntil(db: Db): Promise<Date | null> {
  const row = await db
    .select({ value: appState.value })
    .from(appState)
    .where(eq(appState.key, PAUSED_UNTIL_KEY))
    .get();
  if (!row) return null;
  const until = Number(row.value);
  return Number.isFinite(until) && until > Date.now() ? new Date(until) : null;
}

async function pauseUntil(db: Db, until: Date): Promise<void> {
  const value = String(until.getTime());
  await db
    .insert(appState)
    .values({ key: PAUSED_UNTIL_KEY, value })
    .onConflictDoUpdate({ target: appState.key, set: { value } });
}

export interface SyncRunSummary {
  synced: number;
  /** Projects whose sync failed (including a rate-limited one). */
  errors: number;
  /** Set when GitHub rate-limited the run; later projects were not attempted. */
  rateLimitedUntil: string | null;
  /** True when nothing ran: no token, or a prior rate limit still holds. */
  skipped: "no-token" | "paused" | null;
}

/** Syncs every project whose next sync is due, up to `limit`. Used by the cron. */
export async function runGithubSync(
  db: Db,
  limit = 25,
): Promise<SyncRunSummary> {
  const summary: SyncRunSummary = {
    synced: 0,
    errors: 0,
    rateLimitedUntil: null,
    skipped: null,
  };

  const token = await readToken();
  if (!token) {
    summary.skipped = "no-token";
    return summary;
  }
  const paused = await getPausedUntil(db);
  if (paused) {
    summary.skipped = "paused";
    summary.rateLimitedUntil = paused.toISOString();
    return summary;
  }

  const due = await db
    .select()
    .from(projects)
    .where(
      and(
        isNotNull(projects.githubRepo),
        or(
          isNull(projects.githubNextSyncAt),
          lte(projects.githubNextSyncAt, new Date()),
        ),
      ),
    )
    .orderBy(asc(projects.githubNextSyncAt))
    .limit(limit);

  for (const project of due) {
    let outcome: SyncOutcome;
    try {
      outcome = await syncProject(db, token, project);
    } catch (cause) {
      // A failing write here usually means the DB is unreachable, so the error can't be recorded.
      logger.warn("github sync crashed", {
        repo: project.githubRepo,
        cause: String(cause),
      });
      summary.errors++;
      continue;
    }

    if (outcome.kind === "ok") {
      summary.synced++;
      if (outcome.renamedTo)
        logger.info("github repo renamed", {
          from: project.githubRepo,
          to: outcome.renamedTo,
        });
      continue;
    }

    summary.errors++;
    if (outcome.kind === "rate-limited") {
      const wait = outcome.retryAfterMs ?? RATE_LIMIT_FALLBACK_MS;
      const until = new Date(Date.now() + wait);
      await pauseUntil(db, until);
      summary.rateLimitedUntil = until.toISOString();
      logger.warn("github rate limit hit", {
        waitSeconds: Math.round(wait / 1000),
        processed: summary.synced + summary.errors,
      });
      break;
    }
    logger.warn("github sync failed", {
      repo: project.githubRepo,
      kind: outcome.kind,
      message: outcome.kind === "error" ? outcome.message : undefined,
    });
  }

  return summary;
}

export type ManualSyncResult =
  | {
      kind: "ok";
      sync: {
        githubRepo: string | null;
        githubRepoId: number | null;
        lastGithubActivity: string | null;
        githubSyncedAt: string | null;
        githubSyncError: string | null;
        githubNextSyncAt: string | null;
      };
    }
  | { kind: "no-repo" }
  | { kind: "no-token" }
  | { kind: "not-found" }
  | { kind: "rate-limited"; retryAfterMs: number | null }
  | { kind: "error"; message: string };

/** Syncs one project immediately, regardless of its schedule or any rate-limit pause. */
export async function syncProjectNow(
  db: Db,
  projectId: string,
): Promise<ManualSyncResult | null> {
  const project = await db
    .select()
    .from(projects)
    .where(eq(projects.id, projectId))
    .get();
  if (!project) return null;
  if (!project.githubRepo) return { kind: "no-repo" };
  const token = await readToken();
  if (!token) return { kind: "no-token" };

  let outcome: SyncOutcome;
  try {
    outcome = await syncProject(db, token, project);
  } catch (cause) {
    return {
      kind: "error",
      message: cause instanceof Error ? cause.message : String(cause),
    };
  }

  if (outcome.kind === "rate-limited") {
    await pauseUntil(
      db,
      new Date(Date.now() + (outcome.retryAfterMs ?? RATE_LIMIT_FALLBACK_MS)),
    );
    return { kind: "rate-limited", retryAfterMs: outcome.retryAfterMs };
  }
  if (outcome.kind === "not-found") return { kind: "not-found" };
  if (outcome.kind === "invalid-repo")
    return { kind: "error", message: "Invalid github_repo format" };
  if (outcome.kind === "error")
    return { kind: "error", message: outcome.message };

  const fresh = await db
    .select()
    .from(projects)
    .where(eq(projects.id, projectId))
    .get();
  const iso = (d: Date | null | undefined) => d?.toISOString() ?? null;
  return {
    kind: "ok",
    sync: {
      githubRepo: fresh?.githubRepo ?? null,
      githubRepoId: fresh?.githubRepoId ?? null,
      lastGithubActivity: iso(fresh?.lastGithubActivity),
      githubSyncedAt: iso(fresh?.githubSyncedAt),
      githubSyncError: fresh?.githubSyncError ?? null,
      githubNextSyncAt: iso(fresh?.githubNextSyncAt),
    },
  };
}

/** Clears every schedule so the next runs pick all tracked projects up; returns how many. */
export async function enqueueAllGithubSyncs(db: Db): Promise<number> {
  const rows = await db
    .update(projects)
    .set({ githubNextSyncAt: null })
    .where(isNotNull(projects.githubRepo))
    .returning({ id: projects.id });
  return rows.length;
}

export interface GithubSyncStatus {
  tracked: number;
  neverSynced: number;
  erroring: number;
  due: number;
  nextDueAt: string | null;
  pausedUntil: string | null;
  hasToken: boolean;
}

export async function getGithubSyncStatus(db: Db): Promise<GithubSyncStatus> {
  const nowSeconds = Math.floor(Date.now() / 1000);
  const counts = await db
    .select({
      tracked: sql<number>`count(*)`,
      neverSynced: sql<number>`coalesce(sum(case when ${projects.githubSyncedAt} is null then 1 else 0 end), 0)`,
      erroring: sql<number>`coalesce(sum(case when ${projects.githubSyncError} is not null then 1 else 0 end), 0)`,
      due: sql<number>`coalesce(sum(case when ${projects.githubNextSyncAt} is null or ${projects.githubNextSyncAt} <= ${nowSeconds} then 1 else 0 end), 0)`,
      nextDue: sql<number | null>`min(${projects.githubNextSyncAt})`,
    })
    .from(projects)
    .where(isNotNull(projects.githubRepo))
    .get();
  const paused = await getPausedUntil(db);
  const hasToken = !!(await readToken());
  return {
    tracked: counts?.tracked ?? 0,
    neverSynced: counts?.neverSynced ?? 0,
    erroring: counts?.erroring ?? 0,
    due: counts?.due ?? 0,
    nextDueAt:
      counts?.nextDue != null
        ? new Date(counts.nextDue * 1000).toISOString()
        : null,
    pausedUntil: paused?.toISOString() ?? null,
    hasToken,
  };
}
