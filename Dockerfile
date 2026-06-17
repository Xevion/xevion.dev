# syntax=docker/dockerfile:1.7

# Railway's Dockerfile builder persists BuildKit cache mounts only when the mount
# id is prefixed with `s/<service-id>-`, and it validates that prefix by static
# parse BEFORE build args expand — so the service id must be a literal here, not
# `${RAILWAY_SERVICE_ID}` (the variable form is rejected as "missing the cacheKey
# prefix"; it only "works" in Railpack/local builds that skip this validation).
# This is the `server` service id; it's stable for the service's life.
# https://docs.railway.com/builds/dockerfiles#cache-mounts

# Stage 1: cargo-chef base
FROM rust:1.95-alpine AS chef
WORKDIR /build

RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static && \
    cargo install cargo-chef --locked

# Stage 2: recipe planner
FROM chef AS planner

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: frontend builder
FROM oven/bun:1 AS frontend
WORKDIR /build

# Install system zstd for pre-compression
RUN apt-get update && apt-get install -y zstd && rm -rf /var/lib/apt/lists/*

# Install dependencies (cached until package.json/bun.lock change)
COPY web/package.json web/bun.lock ./
RUN bun install --frozen-lockfile

# Build frontend with environment variables
COPY web/ ./
ARG VITE_OG_R2_BASE_URL
# Public origin baked into prerendered pages (error pages, /pgp) for og:url +
# canonical, since the runtime X-Forwarded-Host fix can't reach static HTML.
# Defaults to https://xevion.dev in code when unset.
ARG VITE_SITE_ORIGIN
RUN bunx svelte-kit sync && bunx panda codegen && bun run build

# Pre-compress static assets (gzip, brotli, zstd)
RUN bun run scripts/compress-assets.ts

# Stage 4: final Rust build (with embedded assets)
FROM chef AS final-builder

# Cook dependencies into the shared cargo target/registry cache mounts. The mount
# ids are keyed by service so Railway persists incremental artifacts across builds.
COPY --from=planner /build/recipe.json recipe.json
RUN --mount=type=cache,id=s/36ae7bd0-8406-42e9-bb51-ba96d9f7261e-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=s/36ae7bd0-8406-42e9-bb51-ba96d9f7261e-cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=s/36ae7bd0-8406-42e9-bb51-ba96d9f7261e-cargo-target,target=/build/target,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

# Copy source
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Copy SQLx offline cache and migrations for compile-time macros
COPY .sqlx/ ./.sqlx/
COPY migrations/ ./migrations/

# Copy frontend assets for embedding
COPY --from=frontend /build/build/client ./web/build/client
COPY --from=frontend /build/build/prerendered ./web/build/prerendered
COPY --from=frontend /build/build/env.js ./web/build/env.js

# Build with real assets (sqlx offline mode). target/ is a cache mount, so the
# compiled binary doesn't land in the layer — copy it out to a real path for the
# runtime stage. Only the server binary ships; the `xevion` CLI is dev-only.
ENV SQLX_OFFLINE=true
RUN --mount=type=cache,id=s/36ae7bd0-8406-42e9-bb51-ba96d9f7261e-cargo-registry,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,id=s/36ae7bd0-8406-42e9-bb51-ba96d9f7261e-cargo-git,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,id=s/36ae7bd0-8406-42e9-bb51-ba96d9f7261e-cargo-target,target=/build/target,sharing=locked \
    cargo build --release --bin xevion-server && \
    cp target/release/xevion-server /build/xevion-server

# Stage 5: runtime
FROM oven/bun:1-alpine AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache ca-certificates tzdata

# Copy Rust server binary
COPY --from=final-builder /build/xevion-server ./xevion-server

# Copy Bun SSR server and client assets (including fonts for OG images)
COPY --from=frontend /build/build/server ./web/build/server
COPY --from=frontend /build/build/client ./web/build/client
COPY --from=frontend /build/build/*.js ./web/build/
COPY web/console-logger.js ./web/

# Install production dependencies for SSR runtime
COPY web/package.json web/bun.lock ./web/
RUN cd web && bun install --frozen-lockfile --production && \
    ln -s /app/web/node_modules /app/web/build/node_modules

# Copy entrypoint script
COPY web/entrypoint.ts ./web/

# Environment configuration
# RUST_LOG - optional, overrides LOG_LEVEL with full tracing filter syntax
# LOG_JSON - defaults to true in Docker, false outside
# PUBLIC_HOSTS - allowlist for per-domain origin/ISR keying; override to add
#   hosts (e.g. "xevion.dev,walters.to"). Without it the proxy runs permissive
#   (HTTP origins), which would emit http:// og:url in production.
ENV PORT=8080 \
    LOG_LEVEL=info \
    LOG_JSON=true \
    PUBLIC_HOSTS=xevion.dev \
    TZ=Etc/UTC

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD wget -q --spider http://localhost:${PORT}/api/health || exit 1

ENTRYPOINT ["bun", "run", "/app/web/entrypoint.ts"]
