import { env } from "cloudflare:workers";
import { getDb } from "./index.ts";
import * as s from "./schema.ts";

export function testDb() {
  return getDb(env.DB);
}

const inDeleteOrder = [
  s.projectTags,
  s.projectRelations,
  s.projectMedia,
  s.projects,
  s.tags,
  s.socialLinks,
  s.sessions,
  s.deviceAuthRequests,
  s.users,
  s.appState,
];

/** Empties every table; nothing rolls back between tests. */
export async function resetDb(): Promise<void> {
  const db = testDb();
  for (const table of inDeleteOrder) await db.delete(table).run();
}
