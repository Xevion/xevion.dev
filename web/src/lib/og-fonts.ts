import { read } from "$app/server";
import { CustomFont, resolveFonts } from "@ethercorps/sveltekit-og/fonts";
import HankenGrotesk900 from "@fontsource/hanken-grotesk/files/hanken-grotesk-latin-900-normal.woff?url";
import SchibstedGrotesk400 from "@fontsource/schibsted-grotesk/files/schibsted-grotesk-latin-400-normal.woff?url";
import Inter500 from "@fontsource/inter/files/inter-latin-500-normal.woff?url";

/**
 * Load fonts for OG image generation.
 * Fonts are sourced from @fontsource packages and imported directly from node_modules.
 * Must be called on each request (fonts can't be cached globally in server context).
 *
 * Note: Only WOFF/TTF/OTF formats are supported by Satori (not WOFF2).
 */
export async function loadOGFonts() {
  const fonts = [
    new CustomFont(
      "Hanken Grotesk",
      () => read(HankenGrotesk900).arrayBuffer(),
      {
        weight: 900,
        style: "normal",
      },
    ),
    new CustomFont(
      "Schibsted Grotesk",
      () => read(SchibstedGrotesk400).arrayBuffer(),
      {
        weight: 400,
        style: "normal",
      },
    ),
    new CustomFont("Inter", () => read(Inter500).arrayBuffer(), {
      weight: 500,
      style: "normal",
    }),
  ];

  return await resolveFonts(fonts);
}
