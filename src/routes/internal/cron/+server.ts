import { apiError, ok } from "#lib/server/api.ts";
import { deleteExpiredSessions } from "#lib/server/sessions.ts";
import { deleteStaleDeviceRequests } from "#lib/server/device-auth.ts";
import { runGithubSync } from "#lib/server/github.ts";
import { ensureOgImages } from "#lib/server/og.ts";
import { getAppLogger } from "#lib/logger.ts";
import type { RequestHandler } from "./$types";

const logger = getAppLogger("cron");

export const POST: RequestHandler = async ({ url, locals }) => {
  const cron = url.searchParams.get("cron");
  const db = locals.db;

  switch (cron) {
    case "*/5 * * * *": {
      const github = await runGithubSync(db);
      logger.info("github sync run", { ...github });
      return ok({ cron, github });
    }
    case "17 9 * * *": {
      const expiredSessions = await deleteExpiredSessions(db);
      const staleDeviceRequests = await deleteStaleDeviceRequests(db);
      await ensureOgImages(db);
      logger.info("daily cleanup", { expiredSessions, staleDeviceRequests });
      return ok({ cron, expiredSessions, staleDeviceRequests });
    }
    default:
      apiError(400, "BAD_REQUEST", `Unknown cron expression: ${cron}`);
  }
};
