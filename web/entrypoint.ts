import { spawn, type Subprocess } from "bun";
import { unlinkSync, existsSync } from "fs";

const BUN_SOCKET = "/tmp/bun.sock";
const API_SOCKET = "/tmp/api.sock";
const PORT = process.env.PORT || "8080";
const LOG_JSON = process.env.LOG_JSON || "true";

function tryUnlink(path: string) {
  try {
    unlinkSync(path);
  } catch (e) {
    // ENOENT is expected (socket doesn't exist yet), other errors are unexpected
    if (e instanceof Error && "code" in e && e.code !== "ENOENT") {
      console.error(`Failed to cleanup ${path}: ${e.message}`);
    }
  }
}

function cleanup() {
  tryUnlink(BUN_SOCKET);
  tryUnlink(API_SOCKET);
}

// Cleanup on signals
process.on("SIGTERM", () => {
  cleanup();
  process.exit(0);
});
process.on("SIGINT", () => {
  cleanup();
  process.exit(0);
});

// Start Bun SSR
console.log("Starting Bun SSR...");
const bunProc = spawn({
  cmd: ["bun", "--preload", "/app/web/console-logger.js", "index.js"],
  cwd: "/app/web/build",
  env: {
    ...process.env,
    SOCKET_PATH: BUN_SOCKET,
    LOG_JSON,
    UPSTREAM_URL: API_SOCKET,
  },
  stdout: "inherit",
  stderr: "inherit",
});

// Wait for Bun socket (5s timeout)
const startTime = Date.now();
while (!existsSync(BUN_SOCKET)) {
  if (Date.now() - startTime > 5000) {
    console.error("ERROR: Bun failed to create socket within 5s");
    bunProc.kill();
    cleanup();
    process.exit(1);
  }
  await Bun.sleep(100);
}

// Start Rust server
console.log("Starting Rust API...");
const rustProc = spawn({
  cmd: [
    "/app/api",
    "--listen",
    `[::]:${PORT}`,
    "--listen",
    API_SOCKET,
    "--downstream",
    BUN_SOCKET,
  ],
  stdout: "inherit",
  stderr: "inherit",
});

// Monitor both processes - exit if either dies
async function monitor(name: string, proc: Subprocess) {
  const exitCode = await proc.exited;
  console.error(`${name} exited with code ${exitCode}`);
  return { name, exitCode };
}

const result = await Promise.race([
  monitor("Bun", bunProc),
  monitor("Rust", rustProc),
]);

// Kill the other process
console.error(`${result.name} died, shutting down...`);
bunProc.kill();
rustProc.kill();
cleanup();
process.exit(result.exitCode || 1);
