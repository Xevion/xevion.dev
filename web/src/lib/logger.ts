import { configure, getConsoleSink, type LogRecord } from "@logtape/logtape";

interface RailwayLogEntry {
  timestamp: string;
  level: string;
  message: string;
  target: string;
  [key: string]: unknown;
}

function railwayFormatter(record: LogRecord): string {
  const entry: RailwayLogEntry = {
    timestamp: new Date().toISOString(),
    level: record.level.toLowerCase(),
    message: record.message.join(" "),
    target: record.category.join(":"),
  };

  if (record.properties && Object.keys(record.properties).length > 0) {
    Object.assign(entry, record.properties);
  }

  return JSON.stringify(entry) + "\n";
}

export async function initLogger() {
  const useJsonLogs =
    process.env.LOG_JSON === "true" || process.env.LOG_JSON === "1";

  const sinkName = useJsonLogs ? "json" : "console";
  const sink = useJsonLogs
    ? (record: LogRecord) => {
        process.stdout.write(railwayFormatter(record));
      }
    : getConsoleSink();

  try {
    await configure({
      sinks: {
        [sinkName]: sink,
      },
      filters: {},
      loggers: [
        {
          category: ["logtape", "meta"],
          lowestLevel: "warning",
          sinks: [sinkName],
        },
        {
          category: [],
          lowestLevel: "debug",
          sinks: [sinkName],
        },
      ],
    });
  } catch (error) {
    if (
      error instanceof Error &&
      error.message.includes("Already configured")
    ) {
      return;
    }
    throw error;
  }
}
