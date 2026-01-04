dev:
    bun run --cwd web dev

setup:
    bun install --cwd web

build:
    bun run --cwd web build

check:
    bun run --cwd web format
    bun run --cwd web lint
    bun run --cwd web check
