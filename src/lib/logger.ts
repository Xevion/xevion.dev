import { dev } from "$app/env";
import { configureSync, getConsoleSink, getLogger } from "@logtape/logtape";

try {
  configureSync({
    sinks: { console: getConsoleSink() },
    loggers: [
      {
        category: ["logtape", "meta"],
        lowestLevel: "warning",
        sinks: ["console"],
      },
      {
        category: ["xevion"],
        lowestLevel: dev ? "debug" : "info",
        sinks: ["console"],
      },
    ],
  });
} catch (error) {
  // Vite HMR re-evaluates this module; a second configure is expected there.
  if (!(error instanceof Error && error.message.includes("Already configured")))
    throw error;
}

export function getAppLogger(...category: string[]) {
  return getLogger(["xevion", ...category]);
}
