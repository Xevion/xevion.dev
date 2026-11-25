import { defineConfig } from "drizzle-kit";

// Only `db:generate` (offline) and `db:studio` use this. Migrations are applied with
// `wrangler d1 migrations apply`, never drizzle-kit, which swallows errors on the d1-http driver.
export default defineConfig({
  dialect: "sqlite",
  schema: "./src/lib/server/db/schema.ts",
  out: "./drizzle",
  driver: "d1-http",
  dbCredentials: {
    accountId: process.env.CLOUDFLARE_ACCOUNT_ID ?? "",
    databaseId: process.env.CLOUDFLARE_DATABASE_ID ?? "",
    token: process.env.CLOUDFLARE_D1_TOKEN ?? "",
  },
});
