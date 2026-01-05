default:
	just --list

dev:
    just dev-json | hl --config .hl.config.toml -P

dev-json:
    LOG_JSON=true UPSTREAM_URL=/tmp/xevion-api.sock bunx concurrently --raw --prefix none "bun run --silent --cwd web dev --port 5173" "cargo watch --quiet --exec 'run --quiet -- --listen localhost:8080 --listen /tmp/xevion-api.sock --downstream http://localhost:5173'"

setup:
    bun install --cwd web
    cargo build

build:
    bun run --cwd web build
    cargo build --release

serve:
    LOG_JSON=true bunx concurrently --raw --prefix none "SOCKET_PATH=/tmp/xevion-bun.sock bun --preload ../console-logger.js --silent --cwd web/build index.js" "target/release/api --listen localhost:8080 --listen /tmp/xevion-api.sock --downstream /tmp/xevion-bun.sock"

check:
    bun run --cwd web format
    bun run --cwd web lint
    bun run --cwd web check
    cargo clippy --all-targets
    cargo fmt --check
