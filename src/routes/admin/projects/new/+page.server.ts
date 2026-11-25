import type { PageServerLoad } from "./$types";
import { apiFetch } from "#lib/api.server.ts";
import type { ApiTagWithCount } from "#lib/bindings/index.ts";

export const load: PageServerLoad = async ({ fetch }) => {
  const result = await apiFetch<ApiTagWithCount[]>("/api/tags", {
    fetch,
  });

  return {
    availableTags: result.unwrapOr([]),
  };
};
