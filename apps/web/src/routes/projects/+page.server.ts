import { getDb } from "$lib/server/db";
import { projects } from "@xevion/db";
import { eq, desc } from "drizzle-orm";
import type { PageServerLoad } from "./$types";

export const load: PageServerLoad = async (event) => {
  const db = getDb(event);

  // Use Drizzle relations for efficient join query
  const projectsWithLinks = await db.query.projects.findMany({
    where: eq(projects.status, "published"),
    orderBy: [desc(projects.updatedAt)],
    with: {
      links: true,
    },
  });

  return {
    projects: projectsWithLinks,
  };
};
