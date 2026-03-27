import type { RequestHandler } from "./$types";
import { apiFetch } from "$lib/api.server";
import { getLogger } from "@logtape/logtape";

const logger = getLogger(["ssr", "routes", "internal", "health"]);

/**
 * Internal health check endpoint.
 * Called by Rust server to validate full round-trip connectivity.
 *
 * IMPORTANT: This endpoint should never be accessible externally.
 * It's blocked by the Rust ISR handler's /internal/* check.
 */
export const GET: RequestHandler = async () => {
  const result = await apiFetch<unknown[]>("/api/projects", {
    signal: AbortSignal.timeout(5000),
  });

  if (result.isErr) {
    logger.error("Health check failed", { error: result.error.message });
    return new Response("Internal health check failed", { status: 503 });
  }

  if (!Array.isArray(result.value)) {
    logger.error("Health check failed: /api/projects returned non-array");
    return new Response("Internal health check failed", { status: 503 });
  }

  logger.debug("Health check passed", { projectCount: result.value.length });
  return new Response("OK", { status: 200 });
};
