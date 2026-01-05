import { getLogger } from "@logtape/logtape";
import { env } from "$env/dynamic/private";

const logger = getLogger(["ssr", "lib", "api"]);

// Compute upstream configuration once at module load
const upstreamUrl = env.UPSTREAM_URL;
const isUnixSocket =
  upstreamUrl?.startsWith("/") || upstreamUrl?.startsWith("./");
const baseUrl = isUnixSocket ? "http://localhost" : upstreamUrl;

/**
 * Fetch utility for calling the Rust backend API.
 * Automatically prefixes requests with the upstream URL from environment.
 * Supports both HTTP URLs and Unix socket paths.
 *
 * Connection pooling and keep-alive are handled automatically by Bun.
 * Default timeout is 30 seconds unless overridden via init.signal.
 */
export async function apiFetch<T>(
  path: string,
  init?: RequestInit,
): Promise<T> {
  if (!upstreamUrl) {
    logger.error("UPSTREAM_URL environment variable not set");
    throw new Error("UPSTREAM_URL environment variable not set");
  }

  const url = `${baseUrl}${path}`;
  const method = init?.method ?? "GET";

  // Build fetch options with 30s default timeout and unix socket support
  const fetchOptions: RequestInit & { unix?: string } = {
    ...init,
    // Respect caller-provided signal, otherwise default to 30s timeout
    signal: init?.signal ?? AbortSignal.timeout(30_000),
  };

  if (isUnixSocket) {
    fetchOptions.unix = upstreamUrl;
  }

  logger.debug("API request", {
    method,
    url,
    path,
    isUnixSocket,
    upstreamUrl,
  });

  try {
    const response = await fetch(url, fetchOptions);

    if (!response.ok) {
      logger.error("API request failed", {
        method,
        url,
        status: response.status,
        statusText: response.statusText,
      });
      throw new Error(`API error: ${response.status} ${response.statusText}`);
    }

    const data = await response.json();
    logger.debug("API response", { method, url, status: response.status });
    return data;
  } catch (error) {
    logger.error("API request exception", {
      method,
      url,
      error: error instanceof Error ? error.message : String(error),
    });
    throw error;
  }
}
