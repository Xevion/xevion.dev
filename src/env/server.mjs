// @ts-check
/**
 * This file is included in `/next.config.mjs` which ensures the app isn't built with invalid env vars.
 * It has to be a `.mjs`-file to be imported there.
 */
import { serverSchema } from "./schema.mjs";
import { env as clientEnv, formatErrors } from "./client.mjs";

const _serverEnv = serverSchema.safeParse(process.env);

if (!_serverEnv.success) {
  console.error(
    "❌ Invalid environment variables:\n",
    ...formatErrors(_serverEnv.error.format()),
  );
  throw new Error("Invalid environment variables");
}

for (let key of Object.keys(_serverEnv.data)) {
  if (key.startsWith("NEXT_PUBLIC_")) {
    console.warn("❌ You are exposing a server-side env-variable:", key);

    throw new Error("You are exposing a server-side env-variable");
  }
}

// Production safety checks
if (process.env.NODE_ENV === "production") {
  if (
    _serverEnv.data.PAYLOAD_SECRET ===
    "dev-secret-change-in-production-immediately"
  ) {
    throw new Error("PAYLOAD_SECRET must be explicitly set in production");
  }
  if (_serverEnv.data.DATABASE_URI?.includes("xevion_dev_password")) {
    throw new Error("DATABASE_URI must be explicitly set in production");
  }
}

// Development warnings for missing optional secrets
if (process.env.NODE_ENV !== "production") {
  const missing = [];
  if (!_serverEnv.data.GITHUB_API_TOKEN) missing.push("GITHUB_API_TOKEN");
  if (!_serverEnv.data.HEALTHCHECK_SECRET) missing.push("HEALTHCHECK_SECRET");
  if (!_serverEnv.data.CRON_SECRET) missing.push("CRON_SECRET");

  if (missing.length > 0) {
    console.warn(`Environment variables missing: [${missing.join(", ")}]`);
  }
}

export const env = { ..._serverEnv.data, ...clientEnv };
