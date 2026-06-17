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

// Prerendered pages (error pages, /pgp) are rendered at build time, where
// SvelteKit's synthetic `http://sveltekit-prerender` origin would otherwise be
// frozen into absolute URLs (og:url, canonical). Substitute the real public
// origin so those static pages carry correct links. SSR pages keep the request
// origin, which the Rust proxy rewrites to the public host via X-Forwarded-Host
// (see XEV-986) — the header fix cannot reach already-prerendered HTML.
const SITE_ORIGIN = import.meta.env.VITE_SITE_ORIGIN ?? "https://xevion.dev";

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

  const origin = building ? SITE_ORIGIN : url.origin;

  return {
    settings,
    // Seeded once per request and serialized into the payload, so relative-time
    // rendering (timeAgo) is identical across SSR and client hydration.
    now: Date.now(),
    // Chosen server-side so the background is in the SSR payload and present on
    // the first paint.
    background: (Math.random() < 0.5 ? "clouds" : "dots") as "clouds" | "dots",
    metadata: {
      title: settings.identity.siteTitle,
      description: settings.identity.bio.split("\n")[0],
      ogImage: getOGImageUrl({ type: "index" }),
      // Query-less canonical: filtered views (e.g. `?tag=`) consolidate to the
      // bare path rather than splitting crawl/share signals.
      url: `${origin}${url.pathname}`,
    },
  };
};
