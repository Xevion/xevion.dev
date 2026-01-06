set dotenv-load

default:
	just --list

[script("bun")]
check:
    const checks = [
      { name: "prettier", cmd: ["bun", "run", "--cwd", "web", "format:check"] },
      { name: "eslint", cmd: ["bun", "run", "--cwd", "web", "lint"] },
      { name: "svelte-check", cmd: ["bun", "run", "--cwd", "web", "check"] },
      { name: "clippy", cmd: ["cargo", "clippy", "--all-targets"] },
      { name: "rustfmt", cmd: ["cargo", "fmt", "--check"] },
    ];

    const start = Date.now();
    const remaining = new Set(checks.map(c => c.name));
    const results = [];

    // Spawn all checks in parallel
    const promises = checks.map(async (check) => {
      const proc = Bun.spawn(check.cmd, {
        env: { ...process.env, FORCE_COLOR: "1" },
        stdout: "pipe",
        stderr: "pipe",
      });

      const [stdout, stderr] = await Promise.all([
        new Response(proc.stdout).text(),
        new Response(proc.stderr).text(),
      ]);

      await proc.exited;
      const elapsed = ((Date.now() - start) / 1000).toFixed(1);
      
      return { ...check, stdout, stderr, exitCode: proc.exitCode, elapsed };
    });

    // Progress updater
    const interval = setInterval(() => {
      const elapsed = ((Date.now() - start) / 1000).toFixed(1);
      const tasks = Array.from(remaining).join(", ");
      process.stderr.write(`\r\x1b[K${elapsed}s [${tasks}]`);
    }, 100);

    // Stream outputs as they complete
    let anyFailed = false;
    for (const promise of promises) {
      const result = await promise;
      remaining.delete(result.name);

      if (result.exitCode !== 0) {
        anyFailed = true;
        process.stderr.write(`\r\x1b[K`);
        process.stdout.write(`❌ ${result.name} (${result.elapsed}s)\n`);
        if (result.stdout) process.stdout.write(result.stdout);
        if (result.stderr) process.stderr.write(result.stderr);
      } else {
        process.stderr.write(`\r\x1b[K`);
        process.stdout.write(`✅ ${result.name} (${result.elapsed}s)\n`);
      }
    }

    clearInterval(interval);
    process.stderr.write(`\r\x1b[K`);
    process.exit(anyFailed ? 1 : 0);

build:
    bun run --cwd web build
    cargo build --release

dev:
    just dev-json | hl --config .hl.config.toml -P

dev-json:
    LOG_JSON=true UPSTREAM_URL=/tmp/xevion-api.sock bunx concurrently --raw --prefix none "bun run --silent --cwd web dev --port 5173" "cargo watch --quiet --exec 'run --quiet -- --listen localhost:8080 --listen /tmp/xevion-api.sock --downstream http://localhost:5173'"

serve:
    just serve-json | hl --config .hl.config.toml -P

serve-json:
    LOG_JSON=true bunx concurrently --raw --prefix none "SOCKET_PATH=/tmp/xevion-bun.sock bun --preload ../console-logger.js --silent --cwd web/build index.js" "target/release/api --listen localhost:8080 --listen /tmp/xevion-api.sock --downstream /tmp/xevion-bun.sock"

docker-image:
    docker build -t xevion-dev .

docker-run port="8080":
	just docker-run-json {{port}} | hl --config .hl.config.toml -P

docker-run-json port="8080":
    #!/usr/bin/env bash
    set -euo pipefail
    docker stop xevion-dev-container 2>/dev/null || true
    docker rm xevion-dev-container 2>/dev/null || true
    # Replace localhost with host.docker.internal for Docker networking
    DOCKER_DATABASE_URL="${DATABASE_URL//localhost/host.docker.internal}"
    docker run --name xevion-dev-container -p {{port}}:8080 --env-file .env -e DATABASE_URL="$DOCKER_DATABASE_URL" xevion-dev

[script("bun")]
seed:
    const { spawnSync } = await import("child_process");
    
    // Ensure DB is running
    const db = spawnSync("just", ["db"], { stdio: "inherit" });
    if (db.status !== 0) process.exit(db.status);
    
    // Run migrations
    const migrate = spawnSync("sqlx", ["migrate", "run"], { stdio: "inherit" });
    if (migrate.status !== 0) process.exit(migrate.status);
    
    // Seed data
    const seed = spawnSync("cargo", ["run", "--bin", "seed"], { stdio: "inherit" });
    if (seed.status !== 0) process.exit(seed.status);
    
    console.log("✅ Database ready with seed data");

[script("bun")]
db cmd="start":
    const fs = await import("fs/promises");
    const { spawnSync } = await import("child_process");
    
    const NAME = "xevion-postgres";
    const USER = "xevion";
    const PASS = "dev";
    const DB = "xevion";
    const PORT = "5432";
    const ENV_FILE = ".env";
    const CMD = "{{cmd}}";
    
    const run = (args) => spawnSync("docker", args, { encoding: "utf8" });
    const getContainer = () => {
      const res = run(["ps", "-a", "--filter", `name=^${NAME}$`, "--format", "json"]);
      return res.stdout.trim() ? JSON.parse(res.stdout) : null;
    };
    
    const updateEnv = async () => {
      const url = `postgresql://${USER}:${PASS}@localhost:${PORT}/${DB}`;
      try {
        let content = await fs.readFile(ENV_FILE, "utf8");
        content = content.includes("DATABASE_URL=") 
          ? content.replace(/DATABASE_URL=.*$/m, `DATABASE_URL=${url}`)
          : content.trim() + `\nDATABASE_URL=${url}\n`;
        await fs.writeFile(ENV_FILE, content);
      } catch {
        await fs.writeFile(ENV_FILE, `DATABASE_URL=${url}\n`);
      }
    };
    
    const create = () => {
      run(["run", "-d", "--name", NAME, "-e", `POSTGRES_USER=${USER}`, 
           "-e", `POSTGRES_PASSWORD=${PASS}`, "-e", `POSTGRES_DB=${DB}`,
           "-p", `${PORT}:5432`, "postgres:16-alpine"]);
      console.log("✅ created");
    };
    
    const container = getContainer();
    
    if (CMD === "rm") {
      if (!container) process.exit(0);
      run(["stop", NAME]);
      run(["rm", NAME]);
      console.log("✅ removed");
    } else if (CMD === "reset") {
      if (!container) create();
      else {
        run(["exec", NAME, "psql", "-U", USER, "-d", "postgres", "-c", `DROP DATABASE IF EXISTS ${DB}`]);
        run(["exec", NAME, "psql", "-U", USER, "-d", "postgres", "-c", `CREATE DATABASE ${DB}`]);
        console.log("✅ reset");
      }
      await updateEnv();
    } else {
      if (!container) {
        create();
      } else if (container.State !== "running") {
        run(["start", NAME]);
        console.log("✅ started");
      } else {
        console.log("✅ running");
      }
      await updateEnv();
    }
