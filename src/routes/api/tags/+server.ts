import { ok, parseJsonBody } from "#lib/server/api.ts";
import {
  createTag,
  listTags,
  TagInputSchema,
  validateTagInput,
} from "#lib/server/tags.ts";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async ({ locals }) =>
  ok(await listTags(locals.db, locals.user !== null));

export const POST: RequestHandler = async ({ request, locals }) => {
  const input = validateTagInput(await parseJsonBody(request, TagInputSchema));
  return ok(await createTag(locals.db, input), { status: 201 });
};
