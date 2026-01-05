import type { Plugin, ViteDevServer } from "vite";
import { configure, getLogger, type LogRecord } from "@logtape/logtape";

interface RailwayLogEntry {
  timestamp: string;
  level: string;
  message: string;
  target: string;
  [key: string]: unknown;
}

/**
 * Railway-compatible JSON formatter for Vite logs
 */
function railwayFormatter(record: LogRecord): string {
  const entry: RailwayLogEntry = {
    timestamp: new Date().toISOString(),
    level: record.level.toLowerCase(),
    message: record.message.join(" "),
    target: "vite",
  };

  // Flatten properties to root level
  if (record.properties && Object.keys(record.properties).length > 0) {
    Object.assign(entry, record.properties);
  }

  return JSON.stringify(entry) + "\n";
}

// Strip ANSI escape codes from strings
function stripAnsi(str: string): string {
  return str.replace(/\u001b\[[0-9;]*m/g, "").trim();
}

export function jsonLogger(): Plugin {
  const useJsonLogs =
    process.env.LOG_JSON === "true" || process.env.LOG_JSON === "1";

  // If JSON logging is disabled, return a minimal plugin that does nothing
  if (!useJsonLogs) {
    return {
      name: "vite-plugin-json-logger",
    };
  }

  // Configure LogTape for Vite plugin logging
  let loggerConfigured = false;
  const configureLogger = async () => {
    if (loggerConfigured) return;
    await configure({
      sinks: {
        json: (record: LogRecord) => {
          process.stdout.write(railwayFormatter(record));
        },
      },
      filters: {},
      loggers: [
        // Suppress LogTape meta logger info messages
        {
          category: ["logtape", "meta"],
          lowestLevel: "warning",
          sinks: ["json"],
        },
        {
          category: ["vite"],
          lowestLevel: "debug",
          sinks: ["json"],
        },
      ],
    });
    loggerConfigured = true;
  };

  let server: ViteDevServer;
  const ignoredMessages = new Set(["press h + enter to show help", "ready in"]);

  return {
    name: "vite-plugin-json-logger",

    async config() {
      await configureLogger();
      const logger = getLogger(["vite"]);

      return {
        customLogger: {
          info(msg: string) {
            const cleaned = stripAnsi(msg);
            // Filter out noise
            if (
              !cleaned ||
              ignoredMessages.has(cleaned) ||
              cleaned.includes("VITE v")
            ) {
              return;
            }
            logger.info(cleaned);
          },
          warn(msg: string) {
            const cleaned = stripAnsi(msg);
            if (cleaned) {
              logger.warn(cleaned);
            }
          },
          error(msg: string) {
            const cleaned = stripAnsi(msg);
            if (cleaned) {
              logger.error(cleaned);
            }
          },
          clearScreen() {
            // No-op since clearScreen is already false
          },
          hasErrorLogged() {
            return false;
          },
          hasWarned: false,
          warnOnce(msg: string) {
            this.warn(msg);
          },
        },
      };
    },

    configureServer(s) {
      server = s;
      const logger = getLogger(["vite"]);

      // Override the default URL printing
      const originalPrintUrls = server.printUrls;
      server.printUrls = () => {
        const urls = server.resolvedUrls;
        if (urls) {
          logger.info("dev server running", {
            local: urls.local,
            network: urls.network,
          });
        }
      };

      // Listen to server events
      server.httpServer?.once("listening", () => {
        logger.info("server listening");
      });

      server.ws.on("connection", () => {
        logger.info("client connected");
      });
    },

    handleHotUpdate({ file, modules }) {
      const logger = getLogger(["vite"]);
      logger.info("hmr update", {
        file: file.replace(process.cwd(), ""),
        modules: modules.length,
      });
      return modules;
    },

    buildStart() {
      const logger = getLogger(["vite"]);
      logger.info("build started");
    },

    buildEnd() {
      const logger = getLogger(["vite"]);
      logger.info("build ended");
    },
  };
}
