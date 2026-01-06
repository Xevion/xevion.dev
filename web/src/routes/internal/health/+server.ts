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
  try {
    // Test connectivity to Rust API by fetching projects
    // Use a 5 second timeout for this health check
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 5000);

    const projects = await apiFetch("/api/projects", {
      signal: controller.signal,
    });

    clearTimeout(timeoutId);

    // Validate response shape
    if (!Array.isArray(projects)) {
      logger.error("Health check failed: /api/projects returned non-array");
      return new Response("Internal health check failed", { status: 503 });
    }

    logger.debug("Health check passed", { projectCount: projects.length });
    return new Response("OK", { status: 200 });
  } catch (error) {
    logger.error("Health check failed", {
      error: error instanceof Error ? error.message : String(error),
    });
    return new Response("Internal health check failed", { status: 503 });
  }
};
