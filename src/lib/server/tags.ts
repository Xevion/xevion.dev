import { type } from "arktype";
import { eq, inArray, sql } from "drizzle-orm";
import type {
  ApiRelatedTag,
  ApiTag,
  ApiTagWithCount,
} from "#lib/bindings/index.ts";
import { apiError } from "./api.ts";
import { chunk } from "./chunk.ts";
import { isUniqueViolation, type Db } from "./db/index.ts";
import { projectTags, projects, tags } from "./db/schema.ts";
import { isHexColor, isUuid, slugify } from "./slug.ts";

export const TagInputSchema = type({
  name: "string",
  "slug?": "string | null",
  "icon?": "string | null",
  "color?": "string | null",
});

type TagRow = typeof tags.$inferSelect;

const byName = sql`${tags.name} collate nocase`;

export function toApiTag(
  row: Pick<TagRow, "id" | "slug" | "name" | "icon" | "color">,
): ApiTag {
  const out: ApiTag = { id: row.id, slug: row.slug, name: row.name };
  if (row.icon) out.icon = row.icon;
  if (row.color) out.color = row.color;
  return out;
}

export function validateTagInput(input: typeof TagInputSchema.infer): {
  name: string;
  slug: string;
  icon: string | null;
  color: string | null;
} {
  const name = input.name.trim();
  if (!name) {
    apiError(400, "VALIDATION_ERROR", "Tag name cannot be empty", {
      name: "Tag name cannot be empty",
    });
  }
  const color = input.color ?? null;
  if (color !== null && !isHexColor(color)) {
    const message =
      "Invalid color format. Must be 6-character hex (e.g., '3b82f6')";
    apiError(400, "VALIDATION_ERROR", message, { color: message });
  }
  const slug = slugify(input.slug?.trim() || name);
  if (!slug) {
    const message = "Slug must contain at least one letter or digit";
    apiError(400, "VALIDATION_ERROR", message, { slug: message });
  }
  return { name, slug, icon: input.icon?.trim() || null, color };
}

const CONFLICT = "A tag with this name or slug already exists";

export async function getTagByRef(
  db: Db,
  ref: string,
): Promise<TagRow | undefined> {
  return db
    .select()
    .from(tags)
    .where(isUuid(ref) ? eq(tags.id, ref) : eq(tags.slug, ref))
    .get();
}

/** Counts cover only projects the viewer may see. */
export async function listTags(
  db: Db,
  admin: boolean,
): Promise<ApiTagWithCount[]> {
  const rows = await db
    .select({
      id: tags.id,
      slug: tags.slug,
      name: tags.name,
      icon: tags.icon,
      color: tags.color,
      projectCount: sql<number>`count(${projects.id})`,
    })
    .from(tags)
    .leftJoin(projectTags, eq(projectTags.tagId, tags.id))
    .leftJoin(
      projects,
      admin
        ? eq(projects.id, projectTags.projectId)
        : sql`${projects.id} = ${projectTags.projectId} and ${projects.hidden} = 0`,
    )
    .groupBy(tags.id)
    .orderBy(byName);
  return rows.map((r) => ({ ...toApiTag(r), projectCount: r.projectCount }));
}

export async function getTagsForProjects(
  db: Db,
  projectIds: string[],
): Promise<Map<string, ApiTag[]>> {
  const result = new Map<string, ApiTag[]>(projectIds.map((id) => [id, []]));
  for (const ids of chunk(projectIds)) {
    const rows = await db
      .select({
        projectId: projectTags.projectId,
        id: tags.id,
        slug: tags.slug,
        name: tags.name,
        icon: tags.icon,
        color: tags.color,
      })
      .from(projectTags)
      .innerJoin(tags, eq(tags.id, projectTags.tagId))
      .where(inArray(projectTags.projectId, ids))
      .orderBy(byName);
    for (const row of rows) result.get(row.projectId)?.push(toApiTag(row));
  }
  return result;
}

export async function getTagsForProject(
  db: Db,
  projectId: string,
): Promise<ApiTag[]> {
  return (await getTagsForProjects(db, [projectId])).get(projectId) ?? [];
}

/** Tags that share the most projects with `tagId`, counting only projects the viewer may see. */
export async function getRelatedTags(
  db: Db,
  tagId: string,
  admin: boolean,
  limit = 10,
): Promise<ApiRelatedTag[]> {
  const visible = admin ? sql`` : sql`and p.hidden = 0`;
  const rows = await db.all<TagRow & { n: number }>(sql`
    select t.id, t.slug, t.name, t.icon, t.color, count(*) as n
    from project_tags a
    join project_tags b on b.project_id = a.project_id and b.tag_id <> a.tag_id
    join projects p on p.id = a.project_id ${visible}
    join tags t on t.id = b.tag_id
    where a.tag_id = ${tagId}
    group by t.id
    order by n desc, t.name collate nocase asc
    limit ${limit}
  `);
  return rows.map((r) => ({
    ...toApiTag(r),
    cooccurrenceCount: r.n,
  }));
}

export async function createTag(
  db: Db,
  input: ReturnType<typeof validateTagInput>,
): Promise<ApiTag> {
  try {
    const row = await db.insert(tags).values(input).returning().get();
    return toApiTag(row);
  } catch (err) {
    if (isUniqueViolation(err)) apiError(409, "CONFLICT", CONFLICT);
    throw err;
  }
}

export async function updateTag(
  db: Db,
  id: string,
  input: ReturnType<typeof validateTagInput>,
): Promise<ApiTag> {
  try {
    const row = await db
      .update(tags)
      .set(input)
      .where(eq(tags.id, id))
      .returning()
      .get();
    return toApiTag(row);
  } catch (err) {
    if (isUniqueViolation(err)) apiError(409, "CONFLICT", CONFLICT);
    throw err;
  }
}

export async function deleteTag(db: Db, id: string): Promise<void> {
  await db.delete(tags).where(eq(tags.id, id)).run();
}

/** Ids from `ids` that exist as tags. */
export async function existingTagIds(
  db: Db,
  ids: string[],
): Promise<Set<string>> {
  const found = new Set<string>();
  for (const part of chunk(ids)) {
    const rows = await db
      .select({ id: tags.id })
      .from(tags)
      .where(inArray(tags.id, part));
    for (const r of rows) found.add(r.id);
  }
  return found;
}
