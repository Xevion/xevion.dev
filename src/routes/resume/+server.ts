import { env } from "cloudflare:workers";
import { error, redirect } from "@sveltejs/kit";
import { getResumeKey } from "#lib/server/settings.ts";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async ({ locals }) => {
  const key = await getResumeKey(locals.db);
  if (!key) error(404, "Resume not available");
  redirect(302, `${env.MEDIA_BASE_URL}/${key}`, { external: true });
};
