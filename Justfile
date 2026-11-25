set dotenv-load

alias c := check
alias d := dev
alias f := format
alias t := test

default:
	just --list

# Type check, lint and format check
check:
	bun run check
	bun run lint

# Apply formatting
format:
	bun run format

# Run the test suite
test:
	bun run test

# Local dev server (local D1, remote-free bindings)
dev:
	bun run dev

# Generate a migration from schema changes
migration:
	bun run db:generate

# Build, boot the built worker under wrangler, and require the homepage to answer 200
smoke:
	#!/usr/bin/env bash
	set -euo pipefail
	bun run build
	bunx wrangler d1 migrations apply xevion-dev --local
	bunx wrangler dev --port 4173 --local &
	pid=$!
	trap 'kill $pid 2>/dev/null || true' EXIT
	for _ in $(seq 1 45); do
		status=$(curl -s -o /dev/null -w '%{http_code}' http://localhost:4173/ || true)
		if [ "$status" = "200" ]; then echo "smoke: / answered 200"; exit 0; fi
		if [ "$status" != "000" ]; then echo "smoke: / answered $status"; exit 1; fi
		sleep 1
	done
	echo "smoke: timed out"; exit 1

# Build, migrate the remote database, and deploy
deploy:
	bun run deploy

# Build and install the `xevion` CLI
install:
	bun run cli:build
	install -m 755 dist/xevion ~/.local/bin/xevion
