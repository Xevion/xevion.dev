# Default environment variables
default_database_url := "postgresql://xevion:xevion@xevion-db:5432/xevion"
default_payload_secret := "development-secret-change-in-production"
network_name := "xevion-net"

# Build the Docker image
docker-build:
    docker build -t xevion.dev .

# Run the Docker container (uses .env if available, otherwise defaults)
docker-run:
    #!/bin/sh
    docker network create {{network_name}} 2>/dev/null || true
    if [ -f .env ]; then
      echo "Loading environment from .env file..."
      docker run -p 3000:3000 \
        --network {{network_name}} \
        -e DATABASE_URL="{{default_database_url}}" \
        --env-file .env \
        xevion.dev | hl -P
    else
      echo "No .env file found, using defaults..."
      docker run -p 3000:3000 \
        --network {{network_name}} \
        -e DATABASE_URL="{{default_database_url}}" \
        -e PAYLOAD_SECRET="{{default_payload_secret}}" \
        xevion.dev | hl -P
    fi

# Start the PostgreSQL database container
docker-db:
    #!/bin/sh
    docker network create {{network_name}} 2>/dev/null || true
    docker run --name xevion-db \
      --network {{network_name}} \
      -p 5432:5432 \
      -e POSTGRES_USER=xevion \
      -e POSTGRES_PASSWORD=xevion \
      -e POSTGRES_DB=xevion \
      -d postgres

# Stop and remove the database container
docker-db-stop:
    docker stop xevion-db && docker rm xevion-db

# Test Docker image with health checks
[script("bun")]
docker-test:
    const $ = async (cmd: string[]) => Bun.spawn(cmd).exited;
    const $out = (cmd: string[]) => Bun.spawnSync(cmd).stdout.toString();

    // Ensure network exists (suppress error if already exists)
    Bun.spawnSync(["docker", "network", "create", "{{network_name}}"], { stderr: "ignore" });

    // Find available port
    const used = new Set(
      $out(["ss", "-tuln"]).split("\n").slice(1)
        .map(l => l.match(/:(\d+)/)?.[1]).filter(Boolean)
    );
    const ranges = [[49152, 65535], [10000, 32767], [5000, 9999]];
    const available = ranges
      .flatMap(([s, e]) => Array.from({ length: e - s + 1 }, (_, i) => s + i))
      .filter(p => !used.has(String(p)));
    const port = available[~~(Math.random() * available.length)];

    console.log(`Using port ${port}`);

    // Start container
    const container = `xevion-test-${port}`;
    const dbUrl = "{{default_database_url}}";
    const secret = "{{default_payload_secret}}";

    await $([
      "docker", "run", "-d", "--name", container,
      "--network", "{{network_name}}",
      "-p", `${port}:3000`,
      "-e", `DATABASE_URL=${dbUrl}`,
      "-e", `PAYLOAD_SECRET=${secret}`,
      "xevion.dev"
    ]);

    const cleanup = async () => {
      console.log("\nCleaning up...");
      await $(["docker", "rm", "-f", container]);
    };
    process.on("SIGINT", async () => { await cleanup(); process.exit(1); });
    process.on("SIGTERM", async () => { await cleanup(); process.exit(1); });

    const base = `http://localhost:${port}`;

    // Poll until success or timeout
    const poll = async (
      fn: () => Promise<boolean>,
      { timeout = 5000, interval = 1000 } = {}
    ): Promise<boolean> => {
      const start = Date.now();
      while (Date.now() - start < timeout) {
        if (await fn()) return true;
        await Bun.sleep(interval);
      }
      return false;
    };

    // Test a route with retries
    const test = async (path: string, expect: { status?: number; contains?: string } = {}) => {
      const { status = 200, contains } = expect;
      let last = { status: 0, reason: "no response" };
      const check = async () => {
        try {
          const res = await fetch(`${base}${path}`);
          const text = await res.text();
          if (res.status === status && (!contains || text.includes(contains))) return true;
          last = { status: res.status, reason: contains && !text.includes(contains) 
            ? `missing "${contains}"` : `status ${res.status}` };
          return false;
        } catch (e: any) { last = { status: 0, reason: e.code || e.message }; return false; }
      };
      const ok = await poll(check);
      console.log(ok ? `  ✓ ${path}` : `  ✗ ${path} (${last.reason})`);
      return ok;
    };

    // Wait for index
    console.log("Waiting for server...");
    const ready = await poll(async () => {
      try { return (await fetch(base)).ok; } catch { return false; }
    }, { timeout: 3000, interval: 100 });

    if (!ready) {
      console.error("Server failed to start");
      await cleanup();
      process.exit(1);
    }
    console.log("Server ready\n");

    // Run tests sequentially
    const tests = [
      () => test("/projects"),
      () => test("/blog"),
      () => test("/admin", { contains: "Payload" }),
      () => test("/admin/api/stats"),
    ];

    const results: boolean[] = [];
    for (const t of tests) results.push(await t());

    await cleanup();
    process.exit(results.every(Boolean) ? 0 : 1);
