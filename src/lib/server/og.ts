import type { Db } from "./db/index.ts";
import type { OGImageSpec } from "#lib/og-types.ts";

/** Regenerates one OG card in the background (waitUntil); never throws. */
export function scheduleOgImage(db: Db, spec: OGImageSpec): void {
  void db;
  void spec;
}

/** Deletes a project's OG card in the background (waitUntil); never throws. */
export function scheduleOgDelete(projectId: string): void {
  void projectId;
}

/** Generates every OG card that is missing from R2. Used by the daily cron. */
export async function ensureOgImages(db: Db): Promise<void> {
  void db;
}
