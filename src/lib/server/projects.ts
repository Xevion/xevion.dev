import { and, desc, eq, inArray, sql } from "drizzle-orm";
import type { BatchItem } from "drizzle-orm/batch";
import type { AdminStats, TerminalCast } from "#lib/bindings/index.ts";
import type { TocItem } from "#lib/tiptap/toc.ts";
import type { Db } from "./db/index.ts";
import { PROJECT_STATUSES, projectTags, projects, tags } from "./db/schema.ts";
import type { ProjectCore } from "./project-mapping.ts";
import { replaceRelationStatements } from "./relations.ts";
import { isUuid } from "./slug.ts";

export type ProjectRow = typeof projects.$inferSelect;

const coreColumns = {
  id: projects.id,
  slug: projects.slug,
  name: projects.name,
  shortDescription: projects.shortDescription,
  status: projects.status,
  hidden: projects.hidden,
  private: projects.private,
  githubRepo: projects.githubRepo,
  demoUrl: projects.demoUrl,
  projectType: projects.projectType,
  accentColor: projects.accentColor,
  lastGithubActivity: projects.lastGithubActivity,
  githubSyncedAt: projects.githubSyncedAt,
  githubSyncError: projects.githubSyncError,
  createdAt: projects.createdAt,
};

const recentFirst = [
  desc(sql`coalesce(${projects.lastGithubActivity}, ${projects.createdAt})`),
  projects.slug,
];

const visibleTo = (admin: boolean) =>
  admin ? undefined : eq(projects.hidden, false);

/** Editorial edits bump `updatedAt`; GitHub sync must not, so this is never a column hook. */
export function touchProjectQuery(db: Db, projectId: string) {
  return db
    .update(projects)
    .set({ updatedAt: new Date() })
    .where(eq(projects.id, projectId));
}

export async function touchProject(db: Db, projectId: string): Promise<void> {
  await touchProjectQuery(db, projectId).run();
}

export async function getProjectRowByRef(
  db: Db,
  ref: string,
): Promise<ProjectRow | undefined> {
  return db
    .select()
    .from(projects)
    .where(isUuid(ref) ? eq(projects.id, ref) : eq(projects.slug, ref))
    .get();
}

/** Hidden projects appear only for admins. Optionally narrowed to one tag. */
export async function listProjectCores(
  db: Db,
  admin: boolean,
  tagId?: string,
): Promise<ProjectCore[]> {
  return db
    .select(coreColumns)
    .from(projects)
    .where(
      and(
        visibleTo(admin),
        tagId
          ? inArray(
              projects.id,
              db
                .select({ id: projectTags.projectId })
                .from(projectTags)
                .where(eq(projectTags.tagId, tagId)),
            )
          : undefined,
      ),
    )
    .orderBy(...recentFirst);
}

export async function getProjectCore(
  db: Db,
  projectId: string,
): Promise<ProjectCore | undefined> {
  return db
    .select(coreColumns)
    .from(projects)
    .where(eq(projects.id, projectId))
    .get();
}

/** Undefined when the project is missing, or hidden from a non-admin viewer. */
export async function getVisibleProjectRow(
  db: Db,
  ref: string,
  admin: boolean,
): Promise<ProjectRow | undefined> {
  const row = await getProjectRowByRef(db, ref);
  return row && (admin || !row.hidden) ? row : undefined;
}

export interface ProjectWrite {
  slug: string;
  name: string;
  shortDescription: string;
  status: (typeof PROJECT_STATUSES)[number];
  hidden: boolean;
  private: boolean;
  githubRepo: string | null;
  githubRepoId: number | null;
  demoUrl: string | null;
  projectType: string | null;
  accentColor: string | null;
  terminalCast: TerminalCast | null;
  detailContent: unknown;
  renderedHtml: string | null;
  toc: TocItem[] | null;
}

function tagStatements(
  db: Db,
  projectId: string,
  tagIds: string[],
): BatchItem<"sqlite">[] {
  const statements: BatchItem<"sqlite">[] = [
    db.delete(projectTags).where(eq(projectTags.projectId, projectId)),
  ];
  if (tagIds.length > 0) {
    statements.push(
      db
        .insert(projectTags)
        .values(tagIds.map((tagId) => ({ projectId, tagId }))),
    );
  }
  return statements;
}

function batch(db: Db, statements: BatchItem<"sqlite">[]) {
  return db.batch(
    statements as [BatchItem<"sqlite">, ...BatchItem<"sqlite">[]],
  );
}

/** Inserts the project with its tags and relations in one atomic batch. */
export async function insertProject(
  db: Db,
  write: ProjectWrite,
  tagIds: string[],
  relatedIds: string[],
): Promise<string> {
  const id = crypto.randomUUID();
  const now = new Date();
  await batch(db, [
    db.insert(projects).values({
      ...write,
      id,
      githubNextSyncAt: null,
      createdAt: now,
      updatedAt: now,
    }),
    ...tagStatements(db, id, tagIds),
    ...replaceRelationStatements(db, id, relatedIds),
  ]);
  return id;
}

/** Replaces every editorial field, the tags and the relations in one atomic batch. */
type ContentField = "detailContent" | "renderedHtml" | "toc";

export async function replaceProject(
  db: Db,
  id: string,
  write: Omit<ProjectWrite, ContentField> &
    Partial<Pick<ProjectWrite, ContentField>>,
  tagIds: string[],
  relatedIds: string[],
  repoChanged: boolean,
): Promise<void> {
  await batch(db, [
    db
      .update(projects)
      .set({
        ...write,
        updatedAt: new Date(),
        ...(repoChanged
          ? {
              githubNextSyncAt: null,
              githubSyncError: null,
              githubErrorCount: 0,
              githubEtag: null,
              githubEtagValue: null,
            }
          : {}),
      })
      .where(eq(projects.id, id)),
    ...tagStatements(db, id, tagIds),
    ...replaceRelationStatements(db, id, relatedIds),
  ]);
}

export async function deleteProject(db: Db, id: string): Promise<void> {
  await db.delete(projects).where(eq(projects.id, id)).run();
}

export async function addTagToProject(
  db: Db,
  projectId: string,
  tagId: string,
): Promise<void> {
  await batch(db, [
    db.insert(projectTags).values({ projectId, tagId }).onConflictDoNothing(),
    touchProjectQuery(db, projectId),
  ]);
}

export async function removeTagFromProject(
  db: Db,
  projectId: string,
  tagId: string,
): Promise<void> {
  await batch(db, [
    db
      .delete(projectTags)
      .where(
        and(eq(projectTags.projectId, projectId), eq(projectTags.tagId, tagId)),
      ),
    touchProjectQuery(db, projectId),
  ]);
}

/** Stores freshly rendered content for a row that was imported without it. */
export async function storeRenderedContent(
  db: Db,
  projectId: string,
  content: {
    detailContent: unknown;
    renderedHtml: string | null;
    toc: TocItem[] | null;
  },
): Promise<void> {
  await db
    .update(projects)
    .set({
      detailContent: content.detailContent,
      renderedHtml: content.renderedHtml,
      toc: content.toc,
    })
    .where(eq(projects.id, projectId))
    .run();
}

export async function getAdminStats(db: Db): Promise<AdminStats> {
  const [byStatus, tagCount] = await Promise.all([
    db
      .select({ status: projects.status, n: sql<number>`count(*)` })
      .from(projects)
      .groupBy(projects.status),
    db
      .select({ n: sql<number>`count(*)` })
      .from(tags)
      .get(),
  ]);
  const projectsByStatus: Record<string, number> = Object.fromEntries(
    PROJECT_STATUSES.map((s) => [s, 0]),
  );
  let totalProjects = 0;
  for (const row of byStatus) {
    projectsByStatus[row.status] = row.n;
    totalProjects += row.n;
  }
  return { totalProjects, projectsByStatus, totalTags: tagCount?.n ?? 0 };
}

export interface SitemapEntry {
  slug: string;
  lastmod: Date;
}

/** Public projects with the newer of the authored edit and the last repo activity. */
export async function listSitemapEntries(db: Db): Promise<SitemapEntry[]> {
  const rows = await db
    .select({
      slug: projects.slug,
      updatedAt: projects.updatedAt,
      lastGithubActivity: projects.lastGithubActivity,
    })
    .from(projects)
    .where(eq(projects.hidden, false));
  return rows
    .map((r) => ({
      slug: r.slug,
      lastmod: new Date(
        Math.max(r.updatedAt.getTime(), r.lastGithubActivity?.getTime() ?? 0),
      ),
    }))
    .sort((a, b) => b.lastmod.getTime() - a.lastmod.getTime());
}
