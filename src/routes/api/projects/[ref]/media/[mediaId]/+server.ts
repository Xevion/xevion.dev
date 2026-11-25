import type { RequestHandler } from "./$types";
import { notFound, ok } from "#lib/server/api.ts";
import { deleteProjectMedia, findProjectByRef } from "#lib/server/media.ts";

export const DELETE: RequestHandler = async ({ locals, params }) => {
  const project = await findProjectByRef(locals.db, params.ref);
  if (!project) return notFound("Project");
  const deleted = await deleteProjectMedia(
    locals.db,
    project.id,
    params.mediaId,
  );
  if (!deleted) return notFound("Media");
  return ok(deleted);
};
