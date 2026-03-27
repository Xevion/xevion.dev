import type { PageServerLoad } from "./$types";
import { apiFetch } from "$lib/api.server";
import type { ApiTagWithCount } from "$lib/bindings";

export const load: PageServerLoad = async ({ fetch }) => {
  const result = await apiFetch<ApiTagWithCount[]>("/api/tags", {
    fetch,
  });

  return {
    availableTags: result.unwrapOr([]),
  };
};
