// @ts-check
import { z } from "zod";

/**
 * Specify your server-side environment variables schema here.
 * This way you can ensure the app isn't built with invalid env vars.
 */
export const serverSchema = z.object({
  CRON_SECRET: z.string().nullish(),
  GITHUB_API_TOKEN: z.string(),
  PAYLOAD_SECRET: z.string(),
  DATABASE_URI: z.string(),
  PAYLOAD_REVALIDATE_KEY: z.string().optional(),
  HEALTHCHECK_SECRET: z.string(),
  NODE_ENV: z.enum(["development", "test", "production"]),
  TITLE: z.preprocess((value) => {
    if (value === undefined || value === "") return null;
    return value;
  }, z.string().nullable()),
});

/**
 * Specify your client-side environment variables schema here.
 * This way you can ensure the app isn't built with invalid env vars.
 * To expose them to the client, prefix them with `NEXT_PUBLIC_`.
 */
export const clientSchema = z.object({
  // NEXT_PUBLIC_CLIENTVAR: z.string(),
});

/**
 * You can't destruct `process.env` as a regular object, so you have to do
 * it manually here. This is because Next.js evaluates this at build time,
 * and only used environment variables are included in the build.
 * @type {{ [k in keyof z.infer<typeof clientSchema>]: z.infer<typeof clientSchema>[k] | undefined }}
 */
export const clientEnv = {
  // NEXT_PUBLIC_CLIENTVAR: process.env.NEXT_PUBLIC_CLIENTVAR,
};
