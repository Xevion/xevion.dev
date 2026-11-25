import { waitUntil } from "cloudflare:workers";
import { ok } from "#lib/server/api.ts";
import { enqueueAllGithubSyncs, runGithubSync } from "#lib/server/github.ts";
import { getAppLogger } from "#lib/logger.ts";
import type { RequestHandler } from "./$types";

const logger = getAppLogger("github");

export const POST: RequestHandler = async ({ locals }) => {
  const db = locals.db;
  const enqueued = await enqueueAllGithubSyncs(db);
  waitUntil(
    runGithubSync(db, 100).catch((cause: unknown) => {
      logger.error("manual github sync failed", { cause: String(cause) });
    }),
  );
  return ok({ enqueued }, { status: 202 });
};
