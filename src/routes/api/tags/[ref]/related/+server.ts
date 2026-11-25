import { notFound, ok } from "#lib/server/api.ts";
import { getRelatedTags, getTagByRef } from "#lib/server/tags.ts";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async ({ params, locals }) => {
  const tag = await getTagByRef(locals.db, params.ref);
  if (!tag) notFound("Tag");
  return ok(await getRelatedTags(locals.db, tag!.id, locals.user !== null));
};
