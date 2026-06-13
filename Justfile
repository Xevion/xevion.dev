set dotenv-load

alias c := check
alias d := dev
alias f := format
alias t := test

default:
	just --list

# Validate all code (parallel checks via tempo)
check *flags:
	tempo check {{flags}}

# Auto-format all code
format *flags:
	tempo fmt {{flags}}

# Generate TypeScript bindings from Rust API types
bindings:
	tempo bindings

# Build and optionally serve. Flags: -s (serve), -d (debug), -n (no-build), -i (install)
build *flags:
	tempo run build {{flags}}

# Start dev servers with pretty log formatting
dev:
	script -q -c "tempo dev | hl --config .hl.config.toml -P --interrupt-ignore-count=0" /dev/null

# Start dev servers with raw JSON output
dev-json:
	tempo dev

# Manage local PostgreSQL container (default: start)
db *flags:
	tempo run db {{flags}}

# Run all tests
test:
	cargo nextest run

# Run database migrations (starts DB container first)
migrate:
	just db
	sqlx migrate run

# Start DB + run migrations + seed test data
seed:
	tempo run seed

# Build Docker image
docker-image:
	tempo run docker-image

# Run Docker container on specified port
docker-run port="8080":
	tempo run docker-run --port {{port}}
