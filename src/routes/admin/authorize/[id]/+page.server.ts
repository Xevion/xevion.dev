import { fail } from "@sveltejs/kit";
import { apiError } from "#lib/server/api.ts";
import {
  approveDeviceAuth,
  denyDeviceAuth,
  peekDeviceAuth,
} from "#lib/server/device-auth.ts";
import { getAppLogger } from "#lib/logger.ts";
import type { Actions, PageServerLoad } from "./$types";

const logger = getAppLogger("auth");

export const load: PageServerLoad = async ({ params, locals }) => {
  const peek = await peekDeviceAuth(locals.db, params.id);
  return {
    request:
      peek.status === "pending"
        ? {
            status: peek.status,
            label: peek.label,
            expiresAt: peek.expiresAt.toISOString(),
          }
        : { status: peek.status },
  };
};

export const actions: Actions = {
  approve: async ({ request, params, locals }) => {
    const user = locals.user;
    if (!user) apiError(401, "UNAUTHORIZED", "authentication required");

    const code = String((await request.formData()).get("code") ?? "");
    if (!code.trim())
      return fail(400, { message: "Enter the code shown in your terminal." });

    const result = await approveDeviceAuth(locals.db, params.id, user.id, code);
    switch (result) {
      case "approved":
        logger.info("cli authorization approved", { username: user.username });
        return { outcome: "approved" as const };
      case "mismatch":
        return fail(400, {
          message: "That code does not match the one in your terminal.",
        });
      case "expired":
        return fail(410, {
          message: "This request has expired. Run the login command again.",
        });
      case "not-pending":
        return fail(409, { message: "This request was already resolved." });
    }
  },

  deny: async ({ params, locals }) => {
    if (!locals.user) apiError(401, "UNAUTHORIZED", "authentication required");
    const denied = await denyDeviceAuth(locals.db, params.id);
    if (!denied)
      return fail(409, { message: "This request was already resolved." });
    return { outcome: "denied" as const };
  },
};
