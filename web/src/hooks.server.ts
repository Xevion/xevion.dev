import type { Handle, HandleServerError } from "@sveltejs/kit";
import { dev } from "$app/environment";
import { initLogger } from "$lib/logger";
import { requestContext } from "$lib/server/context";
import { preCacheCollections } from "$lib/server/icons";
import { getLogger } from "@logtape/logtape";
import { minify } from "html-minifier-terser";

await initLogger();

// Pre-cache icon collections before handling any requests
await preCacheCollections();

const logger = getLogger(["ssr", "error"]);

export const handle: Handle = async ({ event, resolve }) => {
  // Extract request ID from Rust proxy (should always be present in production)
  const requestId = event.request.headers.get("x-request-id");
  if (!requestId) {
    const reqLogger = getLogger(["ssr", "request"]);
    reqLogger.warn(
      "Missing x-request-id header - request not routed through Rust proxy",
      {
        path: event.url.pathname,
      },
    );
  }

  if (
    dev &&
    event.url.pathname === "/.well-known/appspecific/com.chrome.devtools.json"
  ) {
    return new Response(undefined, { status: 404 });
  }

  return requestContext.run({ requestId: requestId ?? undefined }, async () => {
    const response = await resolve(event, {
      transformPageChunk: !dev
        ? ({ html }) =>
            minify(html, {
              collapseBooleanAttributes: true,
              collapseWhitespace: true,
              conservativeCollapse: true,
              decodeEntities: true,
              html5: true,
              ignoreCustomComments: [/^\[/],
              minifyCSS: true,
              minifyJS: true,
              removeAttributeQuotes: true,
              removeComments: true,
              removeOptionalTags: false,
              removeRedundantAttributes: true,
              removeScriptTypeAttributes: true,
              removeStyleLinkTypeAttributes: true,
              sortAttributes: true,
              sortClassName: true,
            })
        : undefined,
    });

    return response;
  });
};

export const handleError: HandleServerError = async ({
  error,
  event,
  status,
  message,
}) => {
  logger.error(message, {
    status,
    method: event.request.method,
    path: event.url.pathname,
    error: error instanceof Error ? error.message : String(error),
  });

  return {
    message: status === 404 ? "Not Found" : "Internal Error",
  };
};
