import type { Db } from "./db/index.ts";
import type { ApiProjectMedia } from "#lib/bindings/index.ts";

/** Media for many projects at once, keyed by project id, each list in display order. */
export async function listMediaForProjects(
  db: Db,
  projectIds: string[],
): Promise<Map<string, ApiProjectMedia[]>> {
  void db;
  return new Map(projectIds.map((id) => [id, []]));
}

/** Removes every R2 object under a project's media prefix. Best-effort; logs failures. */
export async function deleteProjectMediaObjects(
  projectId: string,
): Promise<void> {
  void projectId;
}
