import { cloudflareTest, readD1Migrations } from "@cloudflare/vitest-plugin";
import { defineConfig } from "vitest/config";

// Neither project loads the sveltekit() plugin: tested modules must stay free of $app/* imports.
export default defineConfig({
  test: {
    projects: [
      {
        test: {
          name: "unit",
          environment: "node",
          include: ["src/**/*.test.ts", "cli/**/*.test.ts"],
          exclude: ["src/**/*.db.test.ts"],
        },
      },
      {
        plugins: [
          // Declared here rather than read from wrangler.jsonc so a test can never reach the
          // production database or bucket.
          cloudflareTest(async () => ({
            remoteBindings: false,
            miniflare: {
              compatibilityDate: "2026-11-01",
              compatibilityFlags: ["nodejs_compat"],
              d1Databases: ["DB"],
              bindings: { TEST_MIGRATIONS: await readD1Migrations("drizzle") },
            },
          })),
        ],
        test: {
          name: "db",
          include: ["src/**/*.db.test.ts"],
          setupFiles: ["./src/lib/server/db/apply-migrations.ts"],
          fileParallelism: false,
        },
      },
    ],
  },
});
