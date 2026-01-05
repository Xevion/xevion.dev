import type { Handle, HandleServerError } from "@sveltejs/kit";
import { dev } from "$app/environment";
import { initLogger } from "$lib/logger";
import { getLogger } from "@logtape/logtape";

await initLogger();

const logger = getLogger(["ssr", "error"]);

export const handle: Handle = async ({ event, resolve }) => {
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
