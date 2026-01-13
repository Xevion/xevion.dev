import { getLogger } from "@logtape/logtape";
import { env } from "$env/dynamic/private";
import { requestContext } from "$lib/server/context";

const logger = getLogger(["ssr", "lib", "api"]);

interface FetchOptions extends RequestInit {
  fetch?: typeof fetch;
}

interface BunFetchOptions extends RequestInit {
  unix?: string;
}

/**
 * Create a socket-aware fetch function
 * Automatically handles Unix socket vs TCP based on UPSTREAM_URL
 */
function createSmartFetch(upstreamUrl: string) {
  const isUnixSocket =
    upstreamUrl.startsWith("/") || upstreamUrl.startsWith("./");
  const baseUrl = isUnixSocket ? "http://localhost" : upstreamUrl;

  return async function smartFetch<T>(
    path: string,
    options?: FetchOptions,
  ): Promise<T> {
    const url = `${baseUrl}${path}`;
    const method = options?.method ?? "GET";

    // Unix sockets require Bun's native fetch
    // SvelteKit's fetch doesn't support the 'unix' option
    const fetchFn = isUnixSocket ? fetch : (options?.fetch ?? fetch);

    const fetchOptions: BunFetchOptions = {
      ...options,
      signal: options?.signal ?? AbortSignal.timeout(30_000),
    };

    // Remove custom fetch property from options (not part of standard RequestInit)
    delete (fetchOptions as Record<string, unknown>).fetch;

    // Forward request ID to Rust API
    const ctx = requestContext.getStore();
    if (ctx?.requestId) {
      fetchOptions.headers = {
        ...fetchOptions.headers,
        "x-request-id": ctx.requestId,
      };
    }

    // Add Unix socket path if needed
    if (isUnixSocket) {
      fetchOptions.unix = upstreamUrl;
    }

    logger.debug("API request", {
      method,
      url,
      path,
      isUnixSocket,
    });

    try {
      const response = await fetchFn(url, fetchOptions);

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
  };
}

// Lazy-initialized fetch function (only throws if UPSTREAM_URL is missing when actually used)
let cachedFetch: ReturnType<typeof createSmartFetch> | null = null;

export async function apiFetch<T>(
  path: string,
  options?: FetchOptions,
): Promise<T> {
  if (!cachedFetch) {
    if (!env.UPSTREAM_URL) {
      const error = "UPSTREAM_URL environment variable not set";
      logger.error(error);
      throw new Error(error);
    }
    cachedFetch = createSmartFetch(env.UPSTREAM_URL);
  }
  return cachedFetch(path, options);
}
