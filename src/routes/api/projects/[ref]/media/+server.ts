import type { RequestHandler } from "./$types";
import { apiError, notFound, ok } from "#lib/server/api.ts";
import {
  addProjectMedia,
  findProjectByRef,
  listProjectMedia,
} from "#lib/server/media.ts";

export const GET: RequestHandler = async ({ locals, params }) => {
  const project = await findProjectByRef(locals.db, params.ref);
  if (!project || (project.hidden && !locals.user)) return notFound("Project");
  return ok(await listProjectMedia(locals.db, project.id));
};

export const POST: RequestHandler = async ({ locals, params, request }) => {
  const project = await findProjectByRef(locals.db, params.ref);
  if (!project) return notFound("Project");

  const form = await request.formData().catch(() => null);
  if (!form)
    return apiError(400, "BAD_REQUEST", "Expected multipart/form-data");
  const file = form.get("file") ?? form.get("media");
  if (!(file instanceof File)) {
    return apiError(400, "VALIDATION_ERROR", "No file provided");
  }
  const blurhash = form.get("blurhash");

  const media = await addProjectMedia(locals.db, project.id, {
    file,
    blurhash: typeof blurhash === "string" ? blurhash : null,
  });
  return ok(media, { status: 201 });
};
