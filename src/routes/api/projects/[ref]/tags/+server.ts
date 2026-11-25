import { type } from "arktype";
import { apiError, notFound, ok, parseJsonBody } from "#lib/server/api.ts";
import {
  addTagToProject,
  getProjectRowByRef,
  getVisibleProjectRow,
} from "#lib/server/projects.ts";
import { isUuid } from "#lib/server/slug.ts";
import { getTagByRef, getTagsForProject } from "#lib/server/tags.ts";
import type { RequestHandler } from "./$types";

const AddTagSchema = type({ tagId: "string" });

export const GET: RequestHandler = async ({ params, locals }) => {
  const project = await getVisibleProjectRow(
    locals.db,
    params.ref,
    locals.user !== null,
  );
  if (!project) notFound("Project");
  return ok(await getTagsForProject(locals.db, project!.id));
};

export const POST: RequestHandler = async ({ params, request, locals }) => {
  const project = await getProjectRowByRef(locals.db, params.ref);
  if (!project) notFound("Project");
  const { tagId } = await parseJsonBody(request, AddTagSchema);
  if (!isUuid(tagId)) {
    apiError(400, "VALIDATION_ERROR", "Tag ID must be a valid UUID", {
      tagId: "Tag ID must be a valid UUID",
    });
  }
  const tag = await getTagByRef(locals.db, tagId);
  if (!tag) notFound("Tag");
  await addTagToProject(locals.db, project!.id, tag!.id);
  return ok(await getTagsForProject(locals.db, project!.id), { status: 201 });
};
