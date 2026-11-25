import { type } from "arktype";
import type { RequestHandler } from "./$types";
import { notFound, ok, parseJsonBody } from "#lib/server/api.ts";
import { findProjectByRef, reorderProjectMedia } from "#lib/server/media.ts";

const ReorderBody = type({ mediaIds: "string[]" });

export const PUT: RequestHandler = async ({ locals, params, request }) => {
  const project = await findProjectByRef(locals.db, params.ref);
  if (!project) return notFound("Project");
  const { mediaIds } = await parseJsonBody(request, ReorderBody);
  return ok(await reorderProjectMedia(locals.db, project.id, mediaIds));
};
