import { asc } from "drizzle-orm";
import type { PageServerLoad } from "./$types";
import { projects } from "#lib/server/db/schema.ts";

export const load: PageServerLoad = async ({ locals }) => ({
  projects: await locals.db
    .select({ id: projects.id, name: projects.name, hidden: projects.hidden })
    .from(projects)
    .orderBy(asc(projects.name)),
});
