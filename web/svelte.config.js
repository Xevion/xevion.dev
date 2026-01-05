import adapter from "svelte-adapter-bun";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),

  kit: {
    adapter: adapter({
      out: "build",
      precompress: false,
      serveAssets: false,
    }),
    alias: {
      $components: "src/lib/components",
    },
  },
};

export default config;
