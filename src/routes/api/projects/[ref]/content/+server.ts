import { and, eq, or } from "drizzle-orm";
import { apiError, notFound, ok } from "#lib/server/api.ts";
import { prepareContent } from "#lib/server/content.ts";
import { projects } from "#lib/server/db/schema.ts";
import type { RequestHandler } from "./$types";

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

const EMPTY_DOC = { type: "doc" };

const etagOf = (updatedAt: Date) => `"${updatedAt.getTime()}"`;

function contentResponse(doc: unknown, updatedAt: Date): Response {
  return ok(doc ?? EMPTY_DOC, { headers: { ETag: etagOf(updatedAt) } });
}

function ifMatchSatisfied(header: string, current: string): boolean {
  return header
    .split(",")
    .map((tag) => tag.trim().replace(/^W\//, ""))
    .some((tag) => tag === "*" || tag === current);
}

async function findProject(db: App.Locals["db"], ref: string) {
  return db
    .select({
      id: projects.id,
      hidden: projects.hidden,
      detailContent: projects.detailContent,
      updatedAt: projects.updatedAt,
    })
    .from(projects)
    .where(
      UUID.test(ref)
        ? or(eq(projects.id, ref), eq(projects.slug, ref))
        : eq(projects.slug, ref),
    )
    .get();
}

export const GET: RequestHandler = async ({ params, locals }) => {
  const project = await findProject(locals.db, params.ref);
  if (!project || (project.hidden && locals.user === null))
    return notFound("Project");
  return contentResponse(project.detailContent, project.updatedAt);
};

export const PUT: RequestHandler = async ({ params, locals, request }) => {
  const project = await findProject(locals.db, params.ref);
  if (!project) return notFound("Project");

  const ifMatch = request.headers.get("if-match");
  if (ifMatch !== null && !ifMatchSatisfied(ifMatch, etagOf(project.updatedAt)))
    apiError(
      412,
      "PRECONDITION_FAILED",
      "Content changed since it was loaded; reload and reapply your edit",
    );

  const body: unknown = await request.json().catch(() => undefined);
  if (body === undefined)
    apiError(400, "VALIDATION_ERROR", "Request body must be a JSON document");

  const prepared = await prepareContent(body);
  // updatedAt is stored in whole seconds; the returned ETag must match what a later GET reads.
  const updatedAt = new Date(Math.floor(Date.now() / 1000) * 1000);

  // With If-Match, the write is conditional on the row still carrying the validated timestamp,
  // so a concurrent save between the check above and this update cannot be overwritten.
  const written = await locals.db
    .update(projects)
    .set({
      detailContent: prepared.detailContent,
      renderedHtml: prepared.renderedHtml,
      toc: prepared.toc,
      updatedAt,
    })
    .where(
      ifMatch === null
        ? eq(projects.id, project.id)
        : and(
            eq(projects.id, project.id),
            eq(projects.updatedAt, project.updatedAt),
          ),
    )
    .returning({ id: projects.id });
  if (written.length === 0)
    apiError(
      412,
      "PRECONDITION_FAILED",
      "Content changed since it was loaded; reload and reapply your edit",
    );

  return contentResponse(prepared.detailContent, updatedAt);
};
