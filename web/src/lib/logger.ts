import { configure, getConsoleSink, type LogRecord } from "@logtape/logtape";

interface RailwayLogEntry {
  timestamp: string;
  level: string;
  message: string;
  target: string;
  [key: string]: unknown;
}

/**
 * Custom formatter that outputs Railway-compatible JSON logs.
 * Format: { timestamp, level, message, target, ...attributes }
 *
 * The target field is constructed from the logger category:
 * - ["ssr"] -> "ssr"
 * - ["ssr", "routes"] -> "ssr:routes"
 * - ["ssr", "api", "auth"] -> "ssr:api:auth"
 */
function railwayFormatter(record: LogRecord): string {
  const entry: RailwayLogEntry = {
    timestamp: new Date().toISOString(),
    level: record.level.toLowerCase(),
    message: record.message.join(" "),
    target: record.category.join(":"),
  };

  // Flatten properties to root level (custom attributes)
  if (record.properties && Object.keys(record.properties).length > 0) {
    Object.assign(entry, record.properties);
  }

  return JSON.stringify(entry) + "\n";
}

/**
 * Initialize LogTape with Railway-compatible JSON logging.
 * Only outputs logs when LOG_JSON=true or LOG_JSON=1 is set.
 * Safe to call multiple times (idempotent - will silently skip if already configured).
 */
export async function initLogger() {
  const useJsonLogs =
    process.env.LOG_JSON === "true" || process.env.LOG_JSON === "1";

  try {
    if (!useJsonLogs) {
      // In development, use default console logging with nice formatting
      await configure({
        sinks: {
          console: getConsoleSink(),
        },
        filters: {},
        loggers: [
          {
            category: ["logtape", "meta"],
            lowestLevel: "warning",
            sinks: ["console"],
          },
          {
            category: [],
            lowestLevel: "debug",
            sinks: ["console"],
          },
        ],
      });
      return;
    }

    // In production/JSON mode, use Railway-compatible JSON formatter
    await configure({
      sinks: {
        json: (record: LogRecord) => {
          process.stdout.write(railwayFormatter(record));
        },
      },
      filters: {},
      loggers: [
        // Meta logger for LogTape's internal messages
        {
          category: ["logtape", "meta"],
          lowestLevel: "warning",
          sinks: ["json"],
        },
        // SSR application logs
        {
          category: ["ssr"],
          lowestLevel: "info",
          sinks: ["json"],
        },
      ],
    });
  } catch (error) {
    // Already configured (HMR in dev mode), silently ignore
    if (
      error instanceof Error &&
      error.message.includes("Already configured")
    ) {
      return;
    }
    throw error;
  }
}
