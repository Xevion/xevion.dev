import { env } from "cloudflare:workers";
import { ok, parseJsonBody } from "#lib/server/api.ts";
import { scheduleOgImage } from "#lib/server/og.ts";
import {
  getSettings,
  SettingsInputSchema,
  updateSettings,
} from "#lib/server/settings.ts";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async ({ locals }) =>
  ok(await getSettings(locals.db, env.MEDIA_BASE_URL));

export const PUT: RequestHandler = async ({ request, locals }) => {
  const input = await parseJsonBody(request, SettingsInputSchema);
  const settings = await updateSettings(locals.db, input, env.MEDIA_BASE_URL);
  scheduleOgImage(locals.db, { type: "index" });
  return ok(settings);
};
