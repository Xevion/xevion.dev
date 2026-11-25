import { fail, redirect } from "@sveltejs/kit";
import { createUser, hasAnyUser } from "#lib/server/auth.ts";
import { isUniqueViolation } from "#lib/server/db/index.ts";
import { createSession } from "#lib/server/sessions.ts";
import { setSessionCookie } from "#lib/server/session-cookie.ts";
import { getAppLogger } from "#lib/logger.ts";
import type { Actions, PageServerLoad } from "./$types";

const logger = getAppLogger("auth");

const MIN_PASSWORD_LENGTH = 12;
const MAX_PASSWORD_LENGTH = 256;

export const load: PageServerLoad = ({ locals }) => {
  if (locals.user) redirect(303, "/admin");
  return {};
};

export const actions: Actions = {
  default: async ({ request, cookies, locals }) => {
    const data = await request.formData();
    const username = String(data.get("username") ?? "").trim();
    const password = String(data.get("password") ?? "");
    const confirmPassword = String(data.get("confirmPassword") ?? "");

    const fieldErrors: Record<string, string> = {};
    if (username.length < 3 || username.length > 64 || /\s/.test(username))
      fieldErrors.username = "Use 3 to 64 characters with no spaces.";
    if (password.length < MIN_PASSWORD_LENGTH)
      fieldErrors.password = `Use at least ${MIN_PASSWORD_LENGTH} characters.`;
    else if (password.length > MAX_PASSWORD_LENGTH)
      fieldErrors.password = `Use at most ${MAX_PASSWORD_LENGTH} characters.`;
    if (password !== confirmPassword)
      fieldErrors.confirmPassword = "Passwords do not match.";
    if (Object.keys(fieldErrors).length > 0)
      return fail(400, { username, fieldErrors });

    // The access check ran before this action, so a second tab can land here after the first won.
    if (await hasAnyUser(locals.db))
      return fail(409, {
        username,
        fieldErrors: {},
        message: "Setup has already been completed.",
      });

    let userId: number;
    try {
      userId = await createUser(locals.db, username, password);
    } catch (cause) {
      if (isUniqueViolation(cause))
        return fail(409, {
          username,
          fieldErrors: {},
          message: "Setup has already been completed.",
        });
      throw cause;
    }

    const { token, expiresAt } = await createSession(
      locals.db,
      userId,
      "browser",
    );
    setSessionCookie(cookies, token, expiresAt);
    logger.info("initial account created", { username });
    redirect(303, "/admin");
  },
};
