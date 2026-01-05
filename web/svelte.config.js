import adapter from "svelte-adapter-bun";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  inlineStyleThreshold: 1000,
  kit: {
    adapter: adapter({
      out: "build",
      precompress: {
        brotli: true,
        gzip: true,
        files: ["html", "js", "json", "css", "svg", "xml", "wasm"],
      },
      serveAssets: false,
    }),
    alias: {
      $components: "src/lib/components",
    },
  },
};

export default config;
