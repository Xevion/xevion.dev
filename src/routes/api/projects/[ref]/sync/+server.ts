import { eq, or } from "drizzle-orm";
import { apiError, notFound, ok } from "#lib/server/api.ts";
import { projects } from "#lib/server/db/schema.ts";
import { syncProjectNow } from "#lib/server/github.ts";
import type { RequestHandler } from "./$types";

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export const POST: RequestHandler = async ({ params, locals }) => {
  const project = await locals.db
    .select({ id: projects.id })
    .from(projects)
    .where(
      UUID.test(params.ref)
        ? or(eq(projects.id, params.ref), eq(projects.slug, params.ref))
        : eq(projects.slug, params.ref),
    )
    .get();
  if (!project) return notFound("Project");

  const result = await syncProjectNow(locals.db, project.id);
  if (!result) return notFound("Project");

  switch (result.kind) {
    case "ok":
      return ok(result.sync);
    case "no-repo":
      return apiError(
        400,
        "VALIDATION_ERROR",
        "Project has no GitHub repository to sync",
        { githubRepo: "Project has no GitHub repository to sync" },
      );
    case "not-found":
      return apiError(
        400,
        "VALIDATION_ERROR",
        "GitHub repository not found or inaccessible",
        { githubRepo: "GitHub repository not found or inaccessible" },
      );
    case "no-token":
      return apiError(
        503,
        "SERVICE_UNAVAILABLE",
        "GitHub sync disabled (no token)",
      );
    case "rate-limited":
      return apiError(
        503,
        "SERVICE_UNAVAILABLE",
        "GitHub API rate limit exceeded; try again later",
      );
    case "error":
      return apiError(502, "BAD_GATEWAY", result.message);
  }
};
