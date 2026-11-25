import { asc, eq, inArray } from "drizzle-orm";
import type { BatchItem } from "drizzle-orm/batch";
import type { ApiRelatedProject } from "#lib/bindings/index.ts";
import { chunk } from "./chunk.ts";
import type { Db } from "./db/index.ts";
import { projectRelations, projects } from "./db/schema.ts";

/** Curated related projects in authored order. Anonymous viewers never see hidden ones. */
export async function getRelatedProjects(
  db: Db,
  projectId: string,
  admin: boolean,
): Promise<ApiRelatedProject[]> {
  const rows = await db
    .select({
      id: projects.id,
      slug: projects.slug,
      name: projects.name,
      projectType: projects.projectType,
      accentColor: projects.accentColor,
      hidden: projects.hidden,
    })
    .from(projectRelations)
    .innerJoin(projects, eq(projects.id, projectRelations.relatedProjectId))
    .where(eq(projectRelations.projectId, projectId))
    .orderBy(asc(projectRelations.position));
  return rows
    .filter((r) => admin || !r.hidden)
    .map((r) => {
      const out: ApiRelatedProject = { id: r.id, slug: r.slug, name: r.name };
      if (r.projectType) out.projectType = r.projectType;
      if (r.accentColor) out.accentColor = r.accentColor;
      return out;
    });
}

/** Ids in authored order with duplicates and self-references dropped. */
export function cleanRelatedIds(projectId: string, ids: string[]): string[] {
  return [...new Set(ids)].filter((id) => id !== projectId);
}

/** Statements that replace a project's related list; run them inside a batch. */
export function replaceRelationStatements(
  db: Db,
  projectId: string,
  relatedIds: string[],
): BatchItem<"sqlite">[] {
  const statements: BatchItem<"sqlite">[] = [
    db
      .delete(projectRelations)
      .where(eq(projectRelations.projectId, projectId)),
  ];
  if (relatedIds.length > 0) {
    statements.push(
      db.insert(projectRelations).values(
        relatedIds.map((relatedProjectId, position) => ({
          projectId,
          relatedProjectId,
          position,
        })),
      ),
    );
  }
  return statements;
}

/** Ids from `ids` that exist as projects. */
export async function existingProjectIds(
  db: Db,
  ids: string[],
): Promise<Set<string>> {
  const found = new Set<string>();
  for (const part of chunk(ids)) {
    const rows = await db
      .select({ id: projects.id })
      .from(projects)
      .where(inArray(projects.id, part));
    for (const r of rows) found.add(r.id);
  }
  return found;
}
