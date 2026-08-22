import { existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import type { RunContext } from "@xevion/tempo";
import { defineConfig, presets, task } from "@xevion/tempo";

const port = process.env.PORT || "10237";
const frontendPort = process.env.FRONTEND_PORT || String(Number(port) + 1);

const BINDINGS_DIR = "web/src/lib/bindings";
const RUST_SOURCES = ["src/**/*.rs", "Cargo.toml", "Cargo.lock"];

/** Rewrite the bindings barrel from the generated per-type files. */
function generateBarrel(root: string): number {
  const dir = join(root, BINDINGS_DIR);
  const types = readdirSync(dir)
    .filter((f) => f.endsWith(".ts") && f !== "index.ts")
    .map((f) => f.replace(/\.ts$/, ""))
    .sort();
  writeFileSync(
    join(dir, "index.ts"),
    "// Auto-generated barrel file -- do not edit manually.\n" +
      "// Regenerate with: tempo bindings\n" +
      `${types.map((t) => `export type { ${t} } from "./${t}";`).join("\n")}\n`,
  );
  return types.length;
}

/**
 * Export ts-rs bindings via a temp dir, then diff-copy them into the tree.
 *
 * Unchanged files keep their contents untouched and types deleted from Rust lose their `.ts`, so
 * the barrel always matches what Rust actually exports. A failed build leaves the tree alone.
 */
async function regenerateBindings(ctx: RunContext): Promise<number> {
  const offline = { SQLX_OFFLINE: "true" };
  const build = await ctx.run(["cargo", "test", "--no-run", "--quiet"], { env: offline });
  if (build !== 0) return build;

  const tmp = mkdtempSync(join(tmpdir(), "xevion-bindings-"));
  try {
    const exported = await ctx.run(["cargo", "test", "export_bindings_", "--quiet"], {
      env: { ...offline, TS_RS_EXPORT_DIR: tmp },
    });
    if (exported !== 0) return exported;

    const dir = join(ctx.cwd, BINDINGS_DIR);
    mkdirSync(dir, { recursive: true });

    const fresh = new Set(readdirSync(tmp).filter((f) => f.endsWith(".ts")));
    let changed = 0;
    for (const file of fresh) {
      const next = readFileSync(join(tmp, file));
      const dest = join(dir, file);
      if (existsSync(dest) && Buffer.compare(next, readFileSync(dest)) === 0) continue;
      writeFileSync(dest, next);
      changed++;
    }
    for (const stale of readdirSync(dir)) {
      if (!stale.endsWith(".ts") || stale === "index.ts") continue;
      if (!fresh.has(stale)) {
        rmSync(join(dir, stale));
        changed++;
      }
    }

    ctx.log(`${generateBarrel(ctx.cwd)} types, ${changed} changed`);
    return 0;
  } finally {
    rmSync(tmp, { recursive: true, force: true });
  }
}

const DB_NAME = "xevion";
const DB_USER = "xevion";
const DB_PASS = "dev";
const DB_CONTAINER = "xevion-postgres";

/**
 * Read back whatever host port Docker bound for 5432.
 *
 * It may have been pinned at creation or auto-assigned via `-p 127.0.0.1::5432`, and only Docker
 * knows which.
 */
async function hostPort(ctx: RunContext): Promise<string> {
  const { stdout } = await ctx.capture(["docker", "port", DB_CONTAINER, "5432"]);
  return stdout.trim().split("\n")[0]?.split(":").pop() || "5432";
}

async function updateEnv(ctx: RunContext): Promise<void> {
  const url = `postgresql://${DB_USER}:${DB_PASS}@localhost:${await hostPort(ctx)}/${DB_NAME}`;
  const file = join(ctx.cwd, ".env");
  try {
    const current = await readFile(file, "utf8");
    await writeFile(
      file,
      current.includes("DATABASE_URL=")
        ? current.replace(/DATABASE_URL=.*$/m, `DATABASE_URL=${url}`)
        : `${current.trim()}\nDATABASE_URL=${url}\n`,
    );
  } catch {
    await writeFile(file, `DATABASE_URL=${url}\n`);
  }
}

async function createContainer(ctx: RunContext): Promise<number> {
  const code = await ctx.run([
    "docker", "run", "-d", "--name", DB_CONTAINER,
    "-e", `POSTGRES_USER=${DB_USER}`,
    "-e", `POSTGRES_PASSWORD=${DB_PASS}`,
    "-e", `POSTGRES_DB=${DB_NAME}`,
    "-p", "127.0.0.1::5432",
    "postgres:16-alpine",
  ]);
  ctx.log("created");
  return code;
}

async function inspectContainer(ctx: RunContext): Promise<{ State?: string } | null> {
  const { stdout } = await ctx.capture([
    "docker", "ps", "-a", "--filter", `name=^${DB_CONTAINER}$`, "--format", "json",
  ]);
  return stdout.trim() ? JSON.parse(stdout.trim()) : null;
}

export default defineConfig({
  tasks: [
    task({
      name: "web:deps",
      cwd: "web",
      body: (ctx) => {
        if (!existsSync(join(ctx.cwd, "node_modules"))) {
          ctx.fail("web/node_modules not found -- run `bun install --cwd web` first");
        }
      },
    }),

    // panda codegen and svelte-check both read .svelte-kit/tsconfig.json.
    task({
      name: "web:sync",
      cwd: "web",
      body: "bun run svelte-kit sync",
      needs: ["web:deps"],
      inputs: ["web/svelte.config.js", "web/src/**/*.{svelte,ts}"],
      outputs: ["web/.svelte-kit/tsconfig.json"],
    }),
    task({
      name: "web:codegen",
      cwd: "web",
      body: "bun run codegen",
      needs: ["web:sync"],
      inputs: ["web/panda.config.ts", "web/src/**/*.{svelte,ts}"],
      outputs: ["web/styled-system/**/*.{js,mjs,d.ts}"],
    }),

    // Rust types to frontend TypeScript, diff-copied and pruned.
    task({
      name: "rust:bindings",
      tags: ["codegen"],
      body: regenerateBindings,
      inputs: RUST_SOURCES,
      outputs: [`${BINDINGS_DIR}/index.ts`],
    }),

    // The TipTap node/mark names, plus the inline-code token vocab, as JSON that the Rust drift
    // test (pm::schema_sync) include_str!s. Regenerate when any source the dump script reads changes.
    task({
      name: "web:pm-schema",
      cwd: "web",
      tags: ["codegen"],
      body: "bun run dump-schema",
      needs: ["web:deps"],
      inputs: ["web/src/lib/tiptap/{extensions*,code-tokens}.ts"],
      outputs: ["src/pm_schema.generated.json"],
    }),

    task({
      name: "web:format",
      cwd: "web",
      body: "bunx prettier --check .",
      tags: ["check"],
      needs: ["web:deps"],
    }),
    task({
      name: "web:format-fix",
      cwd: "web",
      body: "bunx prettier --write .",
      tags: ["format"],
      needs: ["web:deps"],
    }),
    task({
      name: "web:lint",
      cwd: "web",
      body: "bun run lint",
      tags: ["check", "lint"],
      needs: ["web:deps"],
    }),
    task({
      name: "web:type-check",
      cwd: "web",
      body: "bun run check --fail-on-warnings",
      tags: ["check"],
      needs: ["web:codegen", "rust:bindings"],
    }),
    task({
      name: "web:test",
      cwd: "web",
      body: "bun run test",
      tags: ["check", "test"],
      needs: ["web:deps"],
    }),
    task({
      name: "web:build",
      cwd: "web",
      body: ["bunx", "--bun", "vite", "build"],
      tags: ["build"],
      needs: ["web:codegen", "rust:bindings"],
      passthrough: true,
    }),

    ...presets.rust({ name: "rust", bin: "xevion" }),
    task({
      name: "rust:sqlx-prepare",
      body: "cargo sqlx prepare --check",
      tags: ["check"],
      requires: [
        { tool: "cargo-sqlx", hint: "run `cargo sqlx prepare` to update offline query data" },
      ],
    }),
    task({
      name: "rust:machete",
      body: "cargo machete",
      tags: ["check"],
      requires: [{ tool: "cargo-machete", hint: "cargo install cargo-machete" }],
    }),

    task({
      name: "ci:zizmor",
      body: "zizmor .github/workflows/",
      tags: ["check"],
      requires: [{ tool: "zizmor" }],
    }),

    // RUSTSEC-2023-0071: rsa timing sidechannel, no fix available (transitive via sqlx).
    task({
      name: "security:audit",
      body: "cargo audit --ignore RUSTSEC-2023-0071",
      tags: ["check"],
      requires: [{ tool: "cargo-audit", hint: "cargo install cargo-audit" }],
    }),

    // Both verify a generated artifact is committed and current, so they diff against git rather
    // than regenerating in place.
    task({
      name: "bindings:verify",
      body: [
        "sh", "-c",
        "SQLX_OFFLINE=true cargo test export_bindings_ && git diff --exit-code web/src/lib/bindings/",
      ],
      tags: ["check"],
    }),
    task({
      name: "bindings:schema-sync",
      body: [
        "sh", "-c",
        "bun run --cwd web dump-schema && git diff --exit-code src/pm_schema.generated.json",
      ],
      tags: ["check"],
      needs: ["web:deps"],
    }),

    task({
      name: "web:dev",
      cwd: "web",
      body: ["bun", "run", "--silent", "dev", "--port", frontendPort],
      tags: ["dev"],
      persistent: true,
      needs: ["web:deps"],
      env: { LOG_JSON: "true", UPSTREAM_URL: `http://localhost:${port}` },
    }),
    task({
      name: "rust:dev-build",
      body: ["cargo", "build", "--bin", "xevion-server", "--quiet"],
      requires: [{ file: ".env", hint: "copy .env.example or set DATABASE_URL" }],
    }),
    task({
      name: "rust:dev",
      body: [
        "./target/debug/xevion-server",
        "--listen", `localhost:${port}`,
        "--listen", "/tmp/xevion-api.sock",
        "--downstream", `http://localhost:${frontendPort}`,
      ],
      tags: ["dev"],
      persistent: true,
      passthrough: true,
      needs: ["rust:dev-build"],
      env: { LOG_JSON: "true", UPSTREAM_URL: "/tmp/xevion-api.sock" },
      // interrupt: cargo rewrites the binary that is currently executing, and the old process
      // would still hold the port when the new one tried to bind.
      watch: {
        paths: ["src", "Cargo.toml", "Cargo.lock", ".env"],
        exts: [".rs", ".toml", ".lock"],
        debounce: 300,
        interrupt: true,
      },
    }),

    task({
      name: "install",
      // The client CLI builds without the `server` feature, so it skips the embedded-frontend
      // `include_dir!` and the entire server module graph: no vite build, no xevion-server.
      body: [
        "cargo", "install", "--path", ".", "--bin", "xevion",
        "--no-default-features", "--force",
      ],
      passthrough: true,
    }),

    task({
      name: "seed",
      body: async (ctx) => {
        for (const cmd of [
          ["sqlx", "migrate", "run"],
          ["cargo", "run", "--bin", "xevion-server", "--", "seed"],
        ]) {
          const code = await ctx.run(cmd);
          if (code !== 0) return code;
        }
        ctx.log("database ready with seed data");
      },
      needs: ["db"],
    }),

    task({
      name: "db",
      passthrough: true,
      requires: [{ tool: "docker" }],
      body: async (ctx) => {
        const sub = ctx.args[0] ?? "start";
        const container = await inspectContainer(ctx);

        if (sub === "rm") {
          if (!container) return 0;
          await ctx.run(["docker", "stop", DB_CONTAINER]);
          await ctx.run(["docker", "rm", DB_CONTAINER]);
          ctx.log("removed");
          return 0;
        }

        if (sub === "reset") {
          if (!container) {
            await createContainer(ctx);
          } else {
            const psql = (sql: string) =>
              ctx.run([
                "docker", "exec", DB_CONTAINER,
                "psql", "-U", DB_USER, "-d", "postgres", "-c", sql,
              ]);
            await psql(`DROP DATABASE IF EXISTS ${DB_NAME}`);
            await psql(`CREATE DATABASE ${DB_NAME}`);
            ctx.log("reset");
          }
          await updateEnv(ctx);
          return 0;
        }

        if (sub !== "start") ctx.fail(`unknown db command "${sub}" (start, reset, rm)`);

        if (!container) {
          await createContainer(ctx);
        } else if (container.State !== "running") {
          if ((await ctx.run(["docker", "start", DB_CONTAINER])) === 0) {
            ctx.log("started");
          } else {
            // Likely pinned to a host port another container now holds; recreate with a
            // Docker-assigned free one.
            ctx.log("existing container failed to start, recreating");
            await ctx.run(["docker", "rm", "-f", DB_CONTAINER]);
            await createContainer(ctx);
          }
        } else {
          ctx.log("running");
        }
        await updateEnv(ctx);
        return 0;
      },
    }),

    task({
      name: "docker:image",
      body: ["docker", "build", "-t", "xevion-dev", "."],
    }),
  ],

  commands: {
    check: { description: "Run every check", tags: ["check"] },
    fmt: { description: "Apply every formatter", tags: ["format"], concurrency: 1 },
    lint: { description: "Lint every subsystem", tags: ["lint"] },
    test: { description: "Every test suite", tags: ["test"] },
    build: { description: "Build the frontend and the Rust binary", tags: ["build"] },
    dev: {
      description: "Frontend and Rust server with watch and reload",
      tags: ["dev"],
      exitBehavior: "first-exits",
    },
    bindings: {
      description: "Regenerate the TypeScript bindings from Rust types via ts-rs",
      tasks: ["rust:bindings"],
    },
    codegen: {
      description: "Regenerate every committed artifact: ts-rs bindings and the PM schema",
      tags: ["codegen"],
    },
    install: {
      description: "Install the `xevion` client CLI to ~/.cargo/bin",
      tasks: ["install"],
      passthrough: true,
    },
    seed: { description: "Start the database, migrate, and seed sample data", tasks: ["seed"] },
    db: {
      description: "Manage the local PostgreSQL container (start | reset | rm)",
      tasks: ["db"],
      passthrough: true,
    },
    "docker-image": { description: "Build the Docker image", tasks: ["docker:image"] },
  },
});
