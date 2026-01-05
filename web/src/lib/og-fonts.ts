import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { cwd } from "node:process";
import type { SatoriOptions } from "satori";

/**
 * Load fonts for OG image generation.
 * Fonts are loaded directly from node_modules using fs.readFile for production compatibility.
 * Must be called on each request (fonts can't be cached globally in server context).
 *
 * Note: Only WOFF/TTF/OTF formats are supported by Satori (not WOFF2).
 */
export async function loadOGFonts(): Promise<SatoriOptions["fonts"]> {
  // In production, the server runs from web/build, so node_modules is at ../node_modules
  // In dev, we're already in web/ directory
  const workingDir = cwd();
  const nodeModulesPath = workingDir.endsWith("/build")
    ? join(workingDir, "..", "node_modules")
    : join(workingDir, "node_modules");

  const [hankenGrotesk, schibstedGrotesk, inter] = await Promise.all([
    readFile(
      join(
        nodeModulesPath,
        "@fontsource/hanken-grotesk/files/hanken-grotesk-latin-900-normal.woff",
      ),
    ),
    readFile(
      join(
        nodeModulesPath,
        "@fontsource/schibsted-grotesk/files/schibsted-grotesk-latin-400-normal.woff",
      ),
    ),
    readFile(
      join(
        nodeModulesPath,
        "@fontsource/inter/files/inter-latin-500-normal.woff",
      ),
    ),
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
