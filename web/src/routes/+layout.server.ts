import type { LayoutServerLoad } from "./$types";
import { getOGImageUrl } from "$lib/og-types";
import { apiFetch } from "$lib/api.server";
import type { SiteSettings } from "$lib/admin-types";

export const load: LayoutServerLoad = async ({ url, fetch }) => {
  // Fetch site settings for all pages
  const settings = await apiFetch<SiteSettings>("/api/settings", { fetch });

  return {
    settings,
    metadata: {
      title: settings.identity.siteTitle,
      description: settings.identity.bio.split("\n")[0], // First line of bio
      ogImage: getOGImageUrl({ type: "index" }),
      url: url.toString(),
    },
  };
};
