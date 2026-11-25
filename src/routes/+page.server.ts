import { listProjects } from "#lib/server/project-views.ts";
import type { PageServerLoad } from "./$types";

export const load: PageServerLoad = async ({ locals, parent }) => {
  const [{ settings }, projects] = await Promise.all([
    parent(),
    listProjects(locals.db, locals.user !== null),
  ]);

  return { projects, socialLinks: settings.socialLinks };
};
