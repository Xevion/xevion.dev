import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";
import Icons from "unplugin-icons/vite";
import { jsonLogger } from "./vite-plugin-json-logger";

export default defineConfig({
  plugins: [jsonLogger(), sveltekit(), Icons({ compiler: "svelte" })],
  clearScreen: false,
  server: {
    fs: {
      allow: ["styled-system"],
    },
  },
});
