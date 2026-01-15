import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import type { AdminTagWithCount } from "$lib/admin-types";

export const load: PageServerLoad = async ({ fetch }) => {
  const availableTags = await apiFetch<AdminTagWithCount[]>("/api/tags", {
    fetch,
  });

  return {
    availableTags,
  };
};
