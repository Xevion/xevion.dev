import { existsSync } from "node:fs";
import { readFile, writeFile } from "node:fs/promises";
import { defineConfig, presets, runners } from "@xevion/tempo";
import { hasTool } from "@xevion/tempo/proc";

const rustPreset = presets.rust({ bin: "xevion" });
const port = process.env.PORT || "10237";
const frontendPort = process.env.FRONTEND_PORT || String(Number(port) + 1);

export default defineConfig({
  subsystems: {
    ci: {
      aliases: ["actions", "gha"],
      commands: {
        zizmor: { cmd: "zizmor .github/workflows/", requires: ["zizmor"] },
      },
    },
    frontend: {
      aliases: ["front", "web", "fe", "f"],
      cwd: "web",
      requires: ["bun"],
      commands: {
        "format-check": "bunx prettier --check .",
        "format-apply": "bunx prettier --write .",
        lint: "bun run lint",
        "type-check": "bun run check --fail-on-warnings",
        build: "bunx --bun vite build",
      },
      autoFix: {
        "format-check": "format-apply",
      },
    },
    rust: {
      ...rustPreset,
      aliases: ["back", "backend", "be", "rs", "b"],
      commands: {
        ...rustPreset.commands,
        "sqlx-prepare": {
          cmd: "cargo sqlx prepare --check",
          requires: [{ tool: "sqlx", hint: "Run `cargo sqlx prepare` to update offline query data" }],
        },
        machete: {
          cmd: "cargo machete",
          requires: [{ tool: "cargo-machete", hint: "Install with `cargo install cargo-machete`" }],
        },
      },
    },
    security: {
      aliases: ["sec", "audit"],
      commands: {
        // RUSTSEC-2023-0071: rsa timing sidechannel, no fix available (transitive via sqlx)
        audit: {
          cmd: "cargo audit --ignore RUSTSEC-2023-0071",
          requires: [{ tool: "cargo-audit", hint: "Install with `cargo install cargo-audit`" }],
        },
      },
    },
    bindings: {
      aliases: ["bind", "ts-rs"],
      commands: {
        verify: {
          cmd: 'bash -c \'SQLX_OFFLINE=true cargo test export_bindings_ && git diff --exit-code web/src/lib/bindings/\'',
        },
      },
    },
  },
  preflights: [
    (ctx) => {
      if (!existsSync("web/node_modules")) {
        ctx.fail("web/node_modules not found -- run `bun install --cwd web` first");
      }
    },
    {
      label: "svelte-kit sync",
      sources: { dir: "web/src", pattern: "**/*.{svelte,ts}" },
      artifacts: { dir: "web/.svelte-kit", pattern: "tsconfig.json" },
      regenerate: "bun run --cwd web svelte-kit sync",
      reason: "panda codegen and svelte-check depend on .svelte-kit/tsconfig.json",
    },
    {
      label: "panda codegen",
      sources: { dir: "web/src", pattern: "**/*.{svelte,ts}" },
      artifacts: { dir: "web/styled-system", pattern: "**/*.{js,mjs,d.ts}" },
      regenerate: "bun run --cwd web codegen",
      reason: "svelte-check depends on styled-system types",
    },
    {
      label: "ts-rs bindings",
      sources: { dir: "src", pattern: "**/*.rs" },
      artifacts: { dir: "web/src/lib/bindings", pattern: "*.ts" },
      regenerate: "SQLX_OFFLINE=true cargo test export_bindings_",
      reason: "frontend imports generated types from Rust structs",
    },
  ],
  hooks: {
    "before:dev": (ctx) => {
      if (!existsSync(".env")) {
        ctx.fail(".env not found -- copy .env.example or create one with DATABASE_URL");
      }
      if (ctx.targets.has("frontend") && !existsSync("web/node_modules")) {
        ctx.fail("web/node_modules not found -- run `bun install --cwd web` first");
      }
      if (ctx.targets.has("rust") && !hasTool("cargo")) {
        ctx.fail("cargo not found -- install Rust toolchain first");
      }
    },
  },
  dev: {
    exitBehavior: "first-exits",
    processes: {
      frontend: {
        type: "unmanaged",
        cmd: ["bun", "run", "--silent", "dev", "--port", frontendPort],
        env: { LOG_JSON: "true", UPSTREAM_URL: `http://localhost:${port}` },
      },
      rust: {
        type: "managed",
        watch: {
          dirs: ["src"],
          exts: [".rs", ".toml"],
          extraPaths: ["Cargo.lock", ".env"],
          debounce: 300,
        },
        build: { cmd: "cargo build --bin xevion --quiet", verbose: false },
        run: {
          cmd: `./target/debug/xevion --listen localhost:${port} --listen /tmp/xevion-api.sock --downstream http://localhost:${frontendPort}`,
          passthrough: true,
        },
        interrupt: true,
        env: { LOG_JSON: "true", UPSTREAM_URL: "/tmp/xevion-api.sock" },
      },
    },
  },
  commands: {
    check: runners.check({ autoFixStrategy: "fix-first" }),
    fmt: runners.sequential("format-apply", { description: "Sequential per-subsystem formatting", autoFixFallback: true }),
    lint: runners.sequential("lint", { description: "Sequential per-subsystem linting" }),
    dev: runners.dev(),
    "pre-commit": runners.preCommit(),
    bindings: {
      description: "Regenerate TypeScript bindings from Rust types via ts-rs",
      run: async ({ run }) => {
        run(["cargo", "test", "export_bindings_"], { env: { SQLX_OFFLINE: "true" } });
        return 0;
      },
    },
    build: {
      description: "Build frontend + Rust binary, optionally serve or install",
      flags: {
        serve: { type: Boolean, alias: "s", description: "Serve after building" },
        debug: { type: Boolean, alias: "d", description: "Debug build (default: release)" },
        "no-build": { type: Boolean, alias: "n", description: "Skip build step" },
        install: { type: Boolean, alias: "i", description: "Install binary via cargo install" },
      },
      run: async (ctx) => {
        const { serve, debug, "no-build": noBuild, install } = ctx.flags;
        const profile = debug ? "debug" : "release";

        if (!noBuild) {
          console.error(ctx.fmt.c.catBlue(`Building frontend${debug ? " (sourcemaps)" : ""}...`));
          ctx.run(
            debug
              ? ["bunx", "--bun", "vite", "build", "--sourcemap"]
              : ["bunx", "--bun", "vite", "build"],
            { cwd: "web" },
          );

          if (!debug) {
            console.error(ctx.fmt.c.catBlue("Pre-compressing assets..."));
            ctx.run(["bun", "run", "scripts/compress-assets.ts"], { cwd: "web" });
          }

          console.error(ctx.fmt.c.catBlue(`Building Rust (${profile})...`));
          const cargoArgs = ["cargo", "build"];
          if (!debug) cargoArgs.push("--release");
          ctx.run(cargoArgs);
        }

        if (install) {
          console.error(ctx.fmt.c.catBlue(`Installing (${profile})...`));
          const installArgs = ["cargo", "install", "--path", "."];
          if (debug) installArgs.push("--debug");
          ctx.run(installArgs);
        }

        if (serve) {
          console.error(ctx.fmt.c.catBlue(`Serving (${profile})...`));
          const servePort = process.env.PORT || "10237";
          ctx.run([
            "script", "-q", "-c",
            `LOG_JSON=true UPSTREAM_URL=/tmp/xevion-api.sock bunx concurrently --raw --prefix none ` +
              `"SOCKET_PATH=/tmp/xevion-bun.sock bun --preload ../console-logger.js --silent --cwd web/build index.js" ` +
              `"target/${profile}/xevion --listen localhost:${servePort} --listen /tmp/xevion-api.sock --downstream /tmp/xevion-bun.sock" ` +
              `| hl --config .hl.config.toml -P --interrupt-ignore-count=0`,
            "/dev/null",
          ]);
        }

        return 0;
      },
    },
    seed: {
      description: "Start DB, run migrations, and seed sample data",
      run: async (ctx) => {
        ctx.run(["tempo", "db"]);
        ctx.run(["sqlx", "migrate", "run"]);
        ctx.run(["cargo", "run", "--bin", "xevion", "--", "seed"]);
        console.error(ctx.fmt.c.catGreen("Database ready with seed data"));
        return 0;
      },
    },
    db: {
      description: "Manage local PostgreSQL container",
      parameters: ["[subcommand]"],
      run: async (ctx) => {
        const cmd = ctx.args[0] || "start";
        const NAME = "xevion-postgres";
        const USER = "xevion";
        const PASS = "dev";
        const DB = "xevion";
        const PORT = "5432";
        const ENV_FILE = ".env";

        const docker = (...args: string[]) =>
          ctx.runPiped(["docker", ...args]).stdout.trim();

        const getContainer = () => {
          const out = docker("ps", "-a", "--filter", `name=^${NAME}$`, "--format", "json");
          return out ? JSON.parse(out) : null;
        };

        const updateEnv = async () => {
          const url = `postgresql://${USER}:${PASS}@localhost:${PORT}/${DB}`;
          try {
            let content = await readFile(ENV_FILE, "utf8");
            content = content.includes("DATABASE_URL=")
              ? content.replace(/DATABASE_URL=.*$/m, `DATABASE_URL=${url}`)
              : content.trim() + `\nDATABASE_URL=${url}\n`;
            await writeFile(ENV_FILE, content);
          } catch {
            await writeFile(ENV_FILE, `DATABASE_URL=${url}\n`);
          }
        };

        const create = () => {
          ctx.run([
            "docker", "run", "-d", "--name", NAME,
            "-e", `POSTGRES_USER=${USER}`,
            "-e", `POSTGRES_PASSWORD=${PASS}`,
            "-e", `POSTGRES_DB=${DB}`,
            "-p", `${PORT}:5432`,
            "postgres:16-alpine",
          ]);
          console.error(ctx.fmt.c.catGreen("created"));
        };

        const container = getContainer();

        if (cmd === "rm") {
          if (!container) return 0;
          ctx.run(["docker", "stop", NAME]);
          ctx.run(["docker", "rm", NAME]);
          console.error(ctx.fmt.c.catGreen("removed"));
          return 0;
        }

        if (cmd === "reset") {
          if (!container) {
            create();
          } else {
            ctx.run(["docker", "exec", NAME, "psql", "-U", USER, "-d", "postgres", "-c", `DROP DATABASE IF EXISTS ${DB}`]);
            ctx.run(["docker", "exec", NAME, "psql", "-U", USER, "-d", "postgres", "-c", `CREATE DATABASE ${DB}`]);
            console.error(ctx.fmt.c.catGreen("reset"));
          }
          await updateEnv();
          return 0;
        }

        // Default: start
        if (!container) {
          create();
        } else if (container.State !== "running") {
          ctx.run(["docker", "start", NAME]);
          console.error(ctx.fmt.c.catGreen("started"));
        } else {
          console.error(ctx.fmt.c.catGreen("running"));
        }
        await updateEnv();
        return 0;
      },
    },
    "docker-image": {
      description: "Build the Docker image",
      run: async ({ run }) => {
        run(["docker", "build", "-t", "xevion-dev", "."]);
        return 0;
      },
    },
    "docker-run": {
      description: "Run the Docker container with hl log formatting",
      flags: {
        port: { type: String, default: "8080", description: "Host port to bind" },
      },
      run: async (ctx) => {
        // Stop and remove any existing container, ignoring failures
        ctx.runPiped(["docker", "stop", "xevion-dev-container"]);
        ctx.runPiped(["docker", "rm", "xevion-dev-container"]);

        const dbUrl = process.env.DATABASE_URL || "";
        const dockerDbUrl = dbUrl.replaceAll("localhost", "host.docker.internal");

        ctx.run([
          "script", "-q", "-c",
          `docker run --name xevion-dev-container -p ${ctx.flags.port}:8080 --env-file .env -e DATABASE_URL="${dockerDbUrl}" xevion-dev ` +
            `| hl --config .hl.config.toml -P --interrupt-ignore-count=0`,
          "/dev/null",
        ]);
        return 0;
      },
    },
  },
});
