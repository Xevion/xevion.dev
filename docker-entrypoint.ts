#!/usr/bin/env bun

/**
 * Production entrypoint for xevion.dev Docker container.
 * Manages environment validation, database migrations, and service orchestration.
 */

import { spawn, type Subprocess } from "bun";

const log = (
  level: "info" | "warn" | "error",
  message: string,
  data?: Record<string, unknown>,
) => {
  const entry = {
    level,
    timestamp: new Date().toISOString(),
    message,
    ...data,
  };
  console.log(JSON.stringify(entry));
};

function validateEnvironment(): void {
  log("info", "starting deployment", { service: "entrypoint" });

  const required = ["DATABASE_URL", "PAYLOAD_SECRET"] as const;
  const optional = ["R2_BUCKET_NAME"] as const;

  for (const key of required) {
    if (!process.env[key]) {
      log("error", "missing required environment variable", {
        service: "entrypoint",
        variable: key,
      });
      process.exit(1);
    }
  }

  const missing: string[] = [];
  for (const key of optional) {
    if (!process.env[key]) {
      missing.push(key);
    }
  }

  if (missing.length > 0) {
    log("warn", "missing optional environment variables", {
      service: "entrypoint",
      variables: missing,
    });
  }

  log("info", "environment validated", { service: "entrypoint" });
}

async function runMigrations(): Promise<void> {
  log("info", "migrations will run on first payload start", {
    service: "entrypoint",
  });
}

function displayConfig(): void {
  const config = {
    service: "entrypoint",
    port: process.env.PORT || "3000",
    nodeEnv: process.env.NODE_ENV || "production",
    r2Bucket: process.env.R2_BUCKET_NAME || null,
    runtime: `Bun ${Bun.version}`,
  };

  log("info", "configuration", config);
}

interface Service {
  name: string;
  proc: Subprocess;
}

async function startServices(): Promise<Service[]> {
  const port = process.env.PORT || "3000";
  const services: Service[] = [];

  const caddy = spawn({
    cmd: ["caddy", "run", "--config", "Caddyfile"],
    cwd: "/app",
    stdout: "pipe",
    stderr: "pipe",
  });
  services.push({ name: "caddy", proc: caddy });
  log("info", "service started", { service: "caddy", pid: caddy.pid, port });

  // Transform Caddy's JSON logs to match our format
  (async () => {
    for await (const chunk of caddy.stderr) {
      const lines = new TextDecoder().decode(chunk).trim().split("\n");
      for (const line of lines) {
        if (!line) continue;
        try {
          const caddyLog = JSON.parse(line);
          log(caddyLog.level || "info", caddyLog.msg || caddyLog.message, {
            service: "caddy",
            logger: caddyLog.logger,
            ...Object.fromEntries(
              Object.entries(caddyLog).filter(
                ([k]) =>
                  !["level", "ts", "msg", "message", "logger"].includes(k),
              ),
            ),
          });
        } catch {
          // Not JSON, log as-is
          log("info", line, { service: "caddy" });
        }
      }
    }
  })();

  const payload = spawn({
    cmd: ["bun", "/app/apps/payload/.next/standalone/apps/payload/server.js"],
    cwd: "/app",
    env: {
      ...process.env,
      PORT: "5001",
      HOSTNAME: "0.0.0.0",
      NEXT_TELEMETRY_DISABLED: "1",
    },
    stdout: "pipe",
    stderr: "pipe",
  });
  services.push({ name: "payload", proc: payload });
  log("info", "service started", {
    service: "payload",
    pid: payload.pid,
    port: 5001,
  });

  // Log stdout and stderr from Next.js
  (async () => {
    for await (const chunk of payload.stdout) {
      const line = new TextDecoder().decode(chunk).trim();
      if (line) log("info", line, { service: "payload" });
    }
  })();
  (async () => {
    for await (const chunk of payload.stderr) {
      const line = new TextDecoder().decode(chunk).trim();
      if (line) log("error", line, { service: "payload" });
    }
  })();

  const web = spawn({
    cmd: ["/app/apps/web/web-server"],
    cwd: "/app",
    env: { ...process.env, PORT: "5000", VERBOSE: "false" },
    stdout: "pipe",
    stderr: "inherit",
  });
  services.push({ name: "web", proc: web });
  log("info", "service started", { service: "web", pid: web.pid, port: 5000 });

  log("info", "all services started", { service: "entrypoint" });

  return services;
}

function setupShutdownHandlers(services: Service[]): void {
  const cleanup = async () => {
    log("info", "shutdown signal received", { service: "entrypoint" });

    for (const service of services) {
      try {
        service.proc.kill();
      } catch {
        // Process already dead
      }
    }

    await Promise.allSettled(services.map((s) => s.proc.exited));

    log("info", "all services stopped", { service: "entrypoint" });
    process.exit(0);
  };

  process.on("SIGTERM", cleanup);
  process.on("SIGINT", cleanup);
}

async function main(): Promise<void> {
  validateEnvironment();
  await runMigrations();
  displayConfig();

  const services = await startServices();
  setupShutdownHandlers(services);

  const results = await Promise.allSettled(services.map((s) => s.proc.exited));

  const failures = results.filter(
    (r) => r.status === "rejected" || r.value !== 0,
  );
  if (failures.length > 0) {
    log("error", "service exited unexpectedly", {
      service: "entrypoint",
      count: failures.length,
    });
    process.exit(1);
  }
}

main().catch((error) => {
  log("error", "fatal error", {
    service: "entrypoint",
    error: error.message,
    stack: error.stack,
  });
  process.exit(1);
});
