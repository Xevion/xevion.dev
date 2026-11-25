import adapter from "@sveltejs/adapter-cloudflare";
import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import Icons from "unplugin-icons/vite";

export default defineConfig({
  plugins: [
    sveltekit({
      adapter: adapter(),
      inlineStyleThreshold: 2000,
      alias: {
        $components: "src/lib/components",
        "styled-system": "./styled-system/*",
      },
      paths: {
        relative: false /* Required for PostHog session replay with SSR */,
      },
      // Keeps the full route list (admin included) out of the public client bundle.
      router: { resolution: "server" },
      version: { pollInterval: 300_000 },
    }),
    Icons({ compiler: "svelte" }),
  ],
  logLevel: "warn",
  clearScreen: false,
  server: {
    fs: {
      allow: ["styled-system"],
    },
    watch: {
      ignored: ["**/.svelte-kit/generated/**", "**/styled-system/**"],
    },
  },
});
