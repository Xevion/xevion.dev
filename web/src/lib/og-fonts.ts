import { read } from "$app/server";
import type { SatoriOptions } from "satori";

// Import fonts as URLs - Vite will copy them to build output
import hankenGroteskUrl from "@fontsource/hanken-grotesk/files/hanken-grotesk-latin-900-normal.woff?url";
import schibstedGroteskUrl from "@fontsource/schibsted-grotesk/files/schibsted-grotesk-latin-400-normal.woff?url";
import interUrl from "@fontsource/inter/files/inter-latin-500-normal.woff?url";

/**
 * Load fonts for OG image generation.
 * Uses SvelteKit's read() to access fonts from build output.
 * Works in all deployment environments (Docker, edge, serverless).
 *
 * Note: Only WOFF/TTF/OTF formats are supported by Satori (not WOFF2).
 */
export async function loadOGFonts(): Promise<SatoriOptions["fonts"]> {
  // Use SvelteKit's read() - works in all deployment environments
  const [hankenGrotesk, schibstedGrotesk, inter] = await Promise.all([
    read(hankenGroteskUrl).arrayBuffer(),
    read(schibstedGroteskUrl).arrayBuffer(),
    read(interUrl).arrayBuffer(),
  ]);

  return [
    {
      name: "Hanken Grotesk",
      data: hankenGrotesk,
      weight: 900,
      style: "normal",
    },
    {
      name: "Schibsted Grotesk",
      data: schibstedGrotesk,
      weight: 400,
      style: "normal",
    },
    {
      name: "Inter",
      data: inter,
      weight: 500,
      style: "normal",
    },
  ];
}
