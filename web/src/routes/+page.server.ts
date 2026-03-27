import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import type { ApiAdminProject } from "$lib/bindings";

export const load: PageServerLoad = async ({ fetch, parent }) => {
  // Get settings from parent layout
  const parentData = await parent();
  const settings = parentData.settings;

  const projects = await apiFetch<ApiAdminProject[]>("/api/projects", {
    fetch,
  });

  return {
    projects,
    socialLinks: settings.socialLinks,
  };
};
