import { fail, redirect } from "@sveltejs/kit";
import { apiError } from "#lib/server/api.ts";
import { listSessions, revokeSession } from "#lib/server/sessions.ts";
import { clearSessionCookie } from "#lib/server/session-cookie.ts";
import type { Actions, PageServerLoad } from "./$types";

export const load: PageServerLoad = async ({ locals }) => {
  const user = locals.user;
  if (!user) apiError(401, "UNAUTHORIZED", "authentication required");
  return { sessions: await listSessions(locals.db, user.id, user.sessionId) };
};

export const actions: Actions = {
  revoke: async ({ request, locals, cookies }) => {
    const user = locals.user;
    if (!user) apiError(401, "UNAUTHORIZED", "authentication required");

    const id = (await request.formData()).get("id");
    if (typeof id !== "string" || !id)
      return fail(400, { message: "Missing session id" });
    if (!(await revokeSession(locals.db, user.id, id)))
      return fail(404, { message: "Session not found" });

    if (id === user.sessionId) {
      clearSessionCookie(cookies);
      redirect(303, "/admin/login");
    }
    return { revoked: id };
  },
};
