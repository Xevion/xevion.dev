import { existsSync } from "node:fs";
import { defineConfig, presets } from "@xevion/tempo";

const rustPreset = presets.rust({ bin: "xevion" });

export default defineConfig({
  subsystems: {
    ci: {
      aliases: ["actions", "gha"],
      commands: {
        zizmor: "zizmor .github/workflows/",
      },
    },
    frontend: {
      aliases: ["front", "web", "fe"],
      cwd: "web",
      commands: {
        "format-check": "bunx prettier --check .",
        "format-apply": "bunx prettier --write .",
        lint: "bun run lint",
        "type-check": "bun run check --fail-on-warnings",
        build: "bunx --bun vite build",
      },
      autoFix: {
        "format-check": "format-apply",
      },
    },
    rust: {
      ...rustPreset,
      aliases: ["back", "backend", "be", "rs"],
      commands: {
        ...rustPreset.commands,
        "sqlx-prepare": {
          cmd: "cargo sqlx prepare --check",
          hint: "Run `cargo sqlx prepare` to update offline query data",
        },
        bindings: {
          cmd: 'SQLX_OFFLINE=true cargo test export_bindings_ 2>/dev/null; true',
          hint: "Regenerates TypeScript bindings from Rust types via ts-rs",
        },
      },
    },
  },
  preflights: [
    (ctx) => {
      if (!existsSync("web/node_modules")) {
        ctx.fail("web/node_modules not found -- run `bun install --cwd web` first");
      }
    },
    {
      label: "panda codegen",
      sources: { dir: "web/src", pattern: "**/*.{svelte,ts}" },
      artifacts: { dir: "web/styled-system", pattern: "**/*.{js,mjs,d.ts}" },
      regenerate: "bun run --cwd web codegen",
      reason: "svelte-check depends on styled-system types",
    },
  ],
  check: {
    autoFixStrategy: "fix-first",
    exclude: ["frontend:format-apply", "rust:format-apply"],
  },
});
