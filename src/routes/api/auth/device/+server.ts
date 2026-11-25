import { type } from "arktype";
import { env } from "cloudflare:workers";
import { apiError, ok } from "#lib/server/api.ts";
import { startDeviceAuth } from "#lib/server/device-auth.ts";
import type { RequestHandler } from "./$types";

const StartBody = type({ "label?": "string" });

export const POST: RequestHandler = async ({ request, locals }) => {
  const raw = (await request.text()).trim();
  let json: unknown = {};
  if (raw) {
    try {
      json = JSON.parse(raw);
    } catch {
      apiError(400, "VALIDATION_ERROR", "Body must be valid JSON");
    }
  }
  const parsed = StartBody(json);
  if (parsed instanceof type.errors)
    apiError(400, "VALIDATION_ERROR", parsed.summary);

  const started = await startDeviceAuth(locals.db, parsed.label);
  return ok({
    id: started.id,
    userCode: started.userCode,
    verificationUrl: `${env.SITE_ORIGIN}/admin/authorize/${started.id}`,
    expiresAt: started.expiresAt.toISOString(),
    interval: started.interval,
  });
};
