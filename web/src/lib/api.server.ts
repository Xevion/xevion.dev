import { getLogger } from "@logtape/logtape";
import { env } from "$env/dynamic/private";

const logger = getLogger(["ssr", "lib", "api"]);

const upstreamUrl = env.UPSTREAM_URL;
const isUnixSocket =
  upstreamUrl?.startsWith("/") || upstreamUrl?.startsWith("./");
const baseUrl = isUnixSocket ? "http://localhost" : upstreamUrl;

export async function apiFetch<T>(
  path: string,
  init?: RequestInit & { fetch?: typeof fetch },
): Promise<T> {
  if (!upstreamUrl) {
    logger.error("UPSTREAM_URL environment variable not set");
    throw new Error("UPSTREAM_URL environment variable not set");
  }

  const url = `${baseUrl}${path}`;
  const method = init?.method ?? "GET";

  // Unix sockets require Bun's native fetch (SvelteKit's fetch doesn't support it)
  const fetchFn = isUnixSocket ? fetch : (init?.fetch ?? fetch);

  const fetchOptions: RequestInit & { unix?: string } = {
    ...init,
    signal: init?.signal ?? AbortSignal.timeout(30_000),
  };

  // Remove custom fetch property from options
  delete (fetchOptions as Record<string, unknown>).fetch;

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
}
