import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import { collectTagIcons } from "$lib/server/tag-icons";
import type { AdminTagWithCount } from "$lib/admin-types";

export const load: PageServerLoad = async ({ fetch }) => {
  const availableTags = await apiFetch<AdminTagWithCount[]>("/api/tags", {
    fetch,
  });

  // Collect icons for sprite
  const icons = await collectTagIcons(availableTags);

  return {
    availableTags,
    icons,
  };
};
