import { apiError, ok } from "#lib/server/api.ts";
import { uploadResume } from "#lib/server/resume.ts";
import type { RequestHandler } from "./$types";

export const POST: RequestHandler = async ({ request, locals }) => {
  const form = await request.formData().catch(() => null);
  const file = form?.get("file");
  if (!(file instanceof File))
    apiError(400, "VALIDATION_ERROR", "No file provided");
  return ok(await uploadResume(locals.db, file as File));
};
