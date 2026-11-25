import { ok } from "#lib/server/api.ts";
import { getAdminStats } from "#lib/server/projects.ts";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async ({ locals }) =>
  ok(await getAdminStats(locals.db));
