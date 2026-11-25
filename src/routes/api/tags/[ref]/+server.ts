import { notFound, ok, parseJsonBody } from "#lib/server/api.ts";
import { listProjects } from "#lib/server/project-views.ts";
import {
  deleteTag,
  getTagByRef,
  TagInputSchema,
  toApiTag,
  updateTag,
  validateTagInput,
} from "#lib/server/tags.ts";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async ({ params, locals }) => {
  const tag = await getTagByRef(locals.db, params.ref);
  if (!tag) notFound("Tag");
  const projects = await listProjects(locals.db, locals.user !== null, tag!.id);
  return ok({ tag: toApiTag(tag!), projects });
};

export const PUT: RequestHandler = async ({ params, request, locals }) => {
  const tag = await getTagByRef(locals.db, params.ref);
  if (!tag) notFound("Tag");
  const input = validateTagInput(await parseJsonBody(request, TagInputSchema));
  return ok(await updateTag(locals.db, tag!.id, input));
};

export const DELETE: RequestHandler = async ({ params, locals }) => {
  const tag = await getTagByRef(locals.db, params.ref);
  if (!tag) notFound("Tag");
  await deleteTag(locals.db, tag!.id);
  return ok(toApiTag(tag!));
};
