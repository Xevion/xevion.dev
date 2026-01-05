set dotenv-load

default:
	just --list

check:
    bun run --cwd web format
    bun run --cwd web lint
    bun run --cwd web check
    cargo clippy --all-targets
    cargo fmt --check

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
