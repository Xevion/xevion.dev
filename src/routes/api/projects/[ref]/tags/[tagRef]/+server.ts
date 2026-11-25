import { notFound, ok } from "#lib/server/api.ts";
import {
  getProjectRowByRef,
  removeTagFromProject,
} from "#lib/server/projects.ts";
import { getTagByRef, getTagsForProject } from "#lib/server/tags.ts";
import type { RequestHandler } from "./$types";

export const DELETE: RequestHandler = async ({ params, locals }) => {
  const project = await getProjectRowByRef(locals.db, params.ref);
  if (!project) notFound("Project");
  const tag = await getTagByRef(locals.db, params.tagRef);
  if (!tag) notFound("Tag");
  await removeTagFromProject(locals.db, project!.id, tag!.id);
  return ok(await getTagsForProject(locals.db, project!.id));
};
