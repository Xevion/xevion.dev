import type { Db } from "./db/index.ts";

/** Accepts `owner/repo` or a pasted GitHub URL; returns `owner/repo` or null when malformed. */
export function normalizeGithubRepo(input: string): string | null {
  void input;
  return null;
}

export type RepoLookup =
  | { kind: "found"; id: number; fullName: string }
  | { kind: "not-found" }
  | { kind: "error"; message: string };

/** GET /repos/{owner}/{repo}, following renames. */
export async function fetchRepo(repo: string): Promise<RepoLookup> {
  void repo;
  return { kind: "error", message: "not implemented" };
}

/** Syncs every project whose next sync is due, up to `limit`. Used by the cron. */
export async function runGithubSync(
  db: Db,
  limit = 25,
): Promise<{ synced: number }> {
  void db;
  void limit;
  return { synced: 0 };
}
