import { read } from "$app/server";
import type { SatoriOptions } from "@cf-wasm/satori";

import hankenGroteskUrl from "@fontsource/hanken-grotesk/files/hanken-grotesk-latin-900-normal.woff?url";
import schibstedGroteskUrl from "@fontsource/schibsted-grotesk/files/schibsted-grotesk-latin-400-normal.woff?url";
import interUrl from "@fontsource/inter/files/inter-latin-500-normal.woff?url";

let cached: Promise<SatoriOptions["fonts"]> | undefined;

/** Font data for OG cards, read once per isolate. Satori reads WOFF but not WOFF2. */
export function loadOGFonts(): Promise<SatoriOptions["fonts"]> {
  cached ??= Promise.all([
    read(hankenGroteskUrl).arrayBuffer(),
    read(schibstedGroteskUrl).arrayBuffer(),
    read(interUrl).arrayBuffer(),
  ]).then(([hankenGrotesk, schibstedGrotesk, inter]) => [
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
    { name: "Inter", data: inter, weight: 500, style: "normal" },
  ]);
  cached.catch(() => {
    cached = undefined;
  });
  return cached;
}
