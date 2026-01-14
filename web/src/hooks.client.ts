import type { HandleClientError } from "@sveltejs/kit";
import { telemetry } from "$lib/telemetry";

export const handleError: HandleClientError = ({ error, status, message }) => {
  telemetry.trackError(
    status >= 500 ? "runtime_error" : "network_error",
    message,
    {
      stack: error instanceof Error ? error.stack : undefined,
      context: { status },
    },
  );

  return {
    message: status === 404 ? "Not Found" : "An error occurred",
  };
};
