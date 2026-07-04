import { error, redirect } from "@sveltejs/kit";
import { apiFetch } from "$lib/api.server";
import type { ApiSiteSettings } from "$lib/bindings";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async ({ fetch }) => {
  const result = await apiFetch<ApiSiteSettings>("/api/settings", { fetch });

  if (result.isErr || !result.value.identity.resumeUrl) {
    error(404, "Resume not available");
  }

  redirect(302, result.value.identity.resumeUrl);
};
