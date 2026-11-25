import { ok, parseJsonBody } from "#lib/server/api.ts";
import {
  createProject,
  ProjectInputSchema,
} from "#lib/server/project-mutations.ts";
import { listProjects } from "#lib/server/project-views.ts";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async ({ locals }) =>
  ok(await listProjects(locals.db, locals.user !== null));

export const POST: RequestHandler = async ({ request, locals }) => {
  const input = await parseJsonBody(request, ProjectInputSchema);
  return ok(await createProject(locals.db, input), { status: 201 });
};
