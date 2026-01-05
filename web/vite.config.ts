import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import Icons from "unplugin-icons/vite";
import { sveltekitOG } from "@ethercorps/sveltekit-og/plugin";
import { jsonLogger } from "./vite-plugin-json-logger";

export default defineConfig({
  plugins: [
    jsonLogger(),
    tailwindcss(),
    sveltekit(),
    sveltekitOG(),
    Icons({ compiler: "svelte" }),
  ],
  clearScreen: false,
  assetsInclude: ["**/*.wasm"],
});
