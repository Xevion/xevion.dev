import { defineConfig } from "vitest/config";
import { fileURLToPath } from "node:url";

/**
 * Standalone Vitest config — deliberately omits the `sveltekit()` plugin so the
 * suite stays a plain Node runner for server-side logic (SSR rendering, pure
 * utilities). Component tests, if added later, would need the Svelte plugin.
 */
export default defineConfig({
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
  },
  resolve: {
    alias: {
      $lib: fileURLToPath(new URL("./src/lib", import.meta.url)),
    },
  },
});
