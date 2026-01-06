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
    docker stop xevion-dev-container 2>/dev/null || true
    docker rm xevion-dev-container 2>/dev/null || true
    docker run --name xevion-dev-container -p {{port}}:8080 xevion-dev
