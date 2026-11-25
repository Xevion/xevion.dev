import type { HandleClientError } from "@sveltejs/kit/hooks";
import posthog from "posthog-js";

export const handleError: HandleClientError = ({ kind, error }) => {
  if (kind === "unknown") posthog.captureException(error);
};
