import adapter from "svelte-adapter-bun";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  inlineStyleThreshold: 2000,
  kit: {
    adapter: adapter({
      out: "build",
      precompress: false,
      serveAssets: false,
    }),
    alias: {
      $components: "src/lib/components",
    },
    paths: {
      relative: false, // Required for PostHog session replay with SSR
    },
    prerender: {
      handleHttpError: ({ path, referrer, message }) => {
        console.log(
          `Prerender error for ${path} (from ${referrer}): ${message}`,
        );
      },
    },
  },
};

export default config;
