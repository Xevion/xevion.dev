import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import type { AdminProject } from "$lib/admin-types";

export const load: PageServerLoad = async ({ fetch }) => {
  const projects = await apiFetch<AdminProject[]>("/api/projects", { fetch });

  return {
    projects,
  };
};
