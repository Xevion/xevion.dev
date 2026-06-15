import type { LayoutServerLoad } from "./$types";
import { getOGImageUrl } from "$lib/og-types";
import { apiFetch } from "$lib/api.server";
import type { ApiSiteSettings } from "$lib/bindings";
import { building } from "$app/environment";

export const trailingSlash = "never";

const DEFAULT_SETTINGS: ApiSiteSettings = {
  identity: {
    siteTitle: "xevion.dev",
    displayName: "Ryan Walters",
    occupation: "Software Engineer",
    bio: "Software engineer and developer",
  },
  socialLinks: [],
};

export const load: LayoutServerLoad = async ({ url, fetch }) => {
  let settings: ApiSiteSettings;

  if (building) {
    // During prerendering, use default settings (API isn't available)
    settings = DEFAULT_SETTINGS;
  } else {
    // At runtime, fetch from API
    const result = await apiFetch<ApiSiteSettings>("/api/settings", { fetch });
    settings = result.unwrapOr(DEFAULT_SETTINGS);
  }

  return {
    settings,
    // Seeded once per request and serialized into the payload, so relative-time
    // rendering (timeAgo) is identical across SSR and client hydration.
    now: Date.now(),
    metadata: {
      title: settings.identity.siteTitle,
      description: settings.identity.bio.split("\n")[0],
      ogImage: getOGImageUrl({ type: "index" }),
      url: url.toString(),
    },
  };
};
