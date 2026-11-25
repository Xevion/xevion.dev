import { ok } from "#lib/server/api.ts";
import { getGithubSyncStatus } from "#lib/server/github.ts";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async ({ locals }) =>
  ok(await getGithubSyncStatus(locals.db));
