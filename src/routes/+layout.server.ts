import { env } from "cloudflare:workers";
import { getOGImageUrl } from "#lib/og-types.ts";
import { getSettings } from "#lib/server/settings.ts";
import type { LayoutServerLoad } from "./$types";

export const trailingSlash = "never";

export const load: LayoutServerLoad = async ({ url, locals }) => {
  const settings = await getSettings(locals.db, env.MEDIA_BASE_URL);

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
      url: `${url.origin}${url.pathname}`,
    },
  };
};
