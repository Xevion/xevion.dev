import type { LayoutServerLoad } from "./$types";
import { getOGImageUrl } from "$lib/og-types";
import { apiFetch } from "$lib/api.server";
import type { SiteSettings } from "$lib/admin-types";
import { building } from "$app/environment";

const DEFAULT_SETTINGS: SiteSettings = {
  identity: {
    siteTitle: "xevion.dev",
    displayName: "Ryan Walters",
    occupation: "Software Engineer",
    bio: "Software engineer and developer",
  },
  socialLinks: [],
};

export const load: LayoutServerLoad = async ({ url, fetch }) => {
  let settings: SiteSettings;

  if (building) {
    // During prerendering, use default settings (API isn't available)
    settings = DEFAULT_SETTINGS;
  } else {
    // At runtime, fetch from API
    settings = await apiFetch<SiteSettings>("/api/settings", { fetch });
  }

  return {
    settings,
    metadata: {
      title: settings.identity.siteTitle,
      description: settings.identity.bio.split("\n")[0],
      ogImage: getOGImageUrl({ type: "index" }),
      url: url.toString(),
    },
  };
};
