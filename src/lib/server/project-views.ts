import type {
  ApiAdminProject,
  ApiProjectMedia,
  ApiRelatedProject,
  ApiTag,
} from "#lib/bindings/index.ts";
import type { Db } from "./db/index.ts";
import { listMediaForProjects } from "./media.ts";
import { toApiAdminProject, type ProjectCore } from "./project-mapping.ts";
import {
  getProjectCore,
  getVisibleProjectRow,
  listProjectCores,
  type ProjectRow,
} from "./projects.ts";
import { getRelatedProjects } from "./relations.ts";
import { getTagsForProject, getTagsForProjects } from "./tags.ts";

async function withTagsAndMedia(
  db: Db,
  cores: ProjectCore[],
): Promise<ApiAdminProject[]> {
  if (cores.length === 0) return [];
  const ids = cores.map((p) => p.id);
  const [tagMap, mediaMap] = await Promise.all([
    getTagsForProjects(db, ids),
    listMediaForProjects(db, ids),
  ]);
  return cores.map((p) =>
    toApiAdminProject(p, tagMap.get(p.id) ?? [], mediaMap.get(p.id) ?? []),
  );
}

/** Newest activity first; hidden projects only for admins; optionally one tag's projects. */
export async function listProjects(
  db: Db,
  admin: boolean,
  tagId?: string,
): Promise<ApiAdminProject[]> {
  return withTagsAndMedia(db, await listProjectCores(db, admin, tagId));
}

export async function getAdminProject(
  db: Db,
  projectId: string,
): Promise<ApiAdminProject | null> {
  const core = await getProjectCore(db, projectId);
  return core ? (await withTagsAndMedia(db, [core]))[0]! : null;
}

export interface ProjectBundle {
  row: ProjectRow;
  tags: ApiTag[];
  media: ApiProjectMedia[];
  related: ApiRelatedProject[];
}

/** Null when the project is missing, or hidden from a non-admin viewer. */
export async function getProjectBundle(
  db: Db,
  ref: string,
  admin: boolean,
): Promise<ProjectBundle | null> {
  const row = await getVisibleProjectRow(db, ref, admin);
  if (!row) return null;
  const [tags, mediaMap, related] = await Promise.all([
    getTagsForProject(db, row.id),
    listMediaForProjects(db, [row.id]),
    getRelatedProjects(db, row.id, admin),
  ]);
  return { row, tags, media: mediaMap.get(row.id) ?? [], related };
}
