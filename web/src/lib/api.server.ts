import { getLogger } from "@logtape/logtape";
import { env } from "$env/dynamic/private";
import { requestContext } from "$lib/server/context";
import { ApiError } from "$lib/errors";
import { Result, ok, err } from "true-myth/result";

const logger = getLogger(["ssr", "lib", "api"]);

interface FetchOptions extends RequestInit {
  fetch?: typeof fetch;
}

interface BunFetchOptions extends RequestInit {
  unix?: string;
}

function createSmartFetch(upstreamUrl: string) {
  const isUnixSocket =
    upstreamUrl.startsWith("/") || upstreamUrl.startsWith("./");
  const baseUrl = isUnixSocket ? "http://localhost" : upstreamUrl;

  return async function smartFetch<T>(
    path: string,
    options?: FetchOptions,
  ): Promise<Result<T, ApiError>> {
    const url = `${baseUrl}${path}`;
    const method = options?.method ?? "GET";

    const fetchFn = isUnixSocket ? fetch : (options?.fetch ?? fetch);

    const fetchOptions: BunFetchOptions = {
      ...options,
      signal: options?.signal ?? AbortSignal.timeout(30_000),
    };

    delete (fetchOptions as Record<string, unknown>).fetch;

    const ctx = requestContext.getStore();
    if (ctx?.requestId) {
      fetchOptions.headers = {
        ...fetchOptions.headers,
        "x-request-id": ctx.requestId,
      };
    }

    if (isUnixSocket) {
      fetchOptions.unix = upstreamUrl;
    }

    logger.debug("API request", { method, url, path, isUnixSocket });

    try {
      const response = await fetchFn(url, fetchOptions);

      if (!response.ok) {
        logger.error("API request failed", {
          method,
          url,
          status: response.status,
          statusText: response.statusText,
        });
        return err(new ApiError(response.status, response.statusText));
      }

      const data = await response.json();
      logger.debug("API response", { method, url, status: response.status });
      return ok(data);
    } catch (error) {
      logger.error("API request exception", {
        method,
        url,
        error: error instanceof Error ? error.message : String(error),
      });
      return err(ApiError.network(error));
    }
  };
}

let cachedFetch: ReturnType<typeof createSmartFetch> | null = null;

export async function apiFetch<T>(
  path: string,
  options?: FetchOptions,
): Promise<Result<T, ApiError>> {
  if (!cachedFetch) {
    if (!env.UPSTREAM_URL) {
      const error = "UPSTREAM_URL environment variable not set";
      logger.error(error);
      return err(new ApiError(500, "Configuration Error", error));
    }
    cachedFetch = createSmartFetch(env.UPSTREAM_URL);
  }
  return cachedFetch(path, options);
}
