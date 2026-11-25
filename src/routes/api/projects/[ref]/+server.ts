import { notFound, ok, parseJsonBody } from "#lib/server/api.ts";
import { toApiProjectDetail } from "#lib/server/project-mapping.ts";
import {
  ProjectInputSchema,
  removeProject,
  updateProject,
} from "#lib/server/project-mutations.ts";
import { getProjectBundle } from "#lib/server/project-views.ts";
import { getProjectRowByRef } from "#lib/server/projects.ts";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async ({ params, locals }) => {
  const bundle = await getProjectBundle(
    locals.db,
    params.ref,
    locals.user !== null,
  );
  if (!bundle) notFound("Project");
  return ok(
    toApiProjectDetail(
      bundle!.row,
      bundle!.tags,
      bundle!.media,
      bundle!.related,
    ),
  );
};

export const PUT: RequestHandler = async ({ params, request, locals }) => {
  const existing = await getProjectRowByRef(locals.db, params.ref);
  if (!existing) notFound("Project");
  const input = await parseJsonBody(request, ProjectInputSchema);
  return ok(await updateProject(locals.db, existing!, input));
};

export const DELETE: RequestHandler = async ({ params, locals }) => {
  const existing = await getProjectRowByRef(locals.db, params.ref);
  if (!existing) notFound("Project");
  return ok(await removeProject(locals.db, existing!));
};
