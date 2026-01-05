import type { Handle, HandleServerError } from "@sveltejs/kit";
import { dev } from "$app/environment";
import { initLogger } from "$lib/logger";
import { getLogger } from "@logtape/logtape";

// Initialize logger on server startup
await initLogger();

const logger = getLogger(["ssr", "error"]);

export const handle: Handle = async ({ event, resolve }) => {
  // Handle DevTools request silently to prevent console.log spam
  if (
    dev &&
    event.url.pathname === "/.well-known/appspecific/com.chrome.devtools.json"
  ) {
    return new Response(undefined, { status: 404 });
  }

  return await resolve(event);
};

export const handleError: HandleServerError = async ({
  error,
  event,
  status,
  message,
}) => {
  // Use structured logging via LogTape instead of console.error
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
