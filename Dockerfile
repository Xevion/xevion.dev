# ========== Stage 1: Cargo Chef Base ==========
FROM rust:1.91-alpine AS chef
WORKDIR /build

RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static && \
    cargo install cargo-chef --locked

# ========== Stage 2: Recipe Planner ==========
FROM chef AS planner

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

RUN cargo chef prepare --recipe-path recipe.json

# ========== Stage 3: Rust Builder ==========
FROM chef AS builder

# Cook dependencies (cached until Cargo.toml/Cargo.lock change)
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source and build
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Create placeholder for embedded assets (will be replaced in final build)
RUN mkdir -p web/build/client && \
    echo "placeholder" > web/build/client/.gitkeep

RUN cargo build --release && \
    strip target/release/api

# ========== Stage 4: Frontend Builder ==========
FROM oven/bun:1 AS frontend
WORKDIR /build

# Install dependencies (cached until package.json/bun.lock change)
COPY web/package.json web/bun.lock ./
RUN bun install --frozen-lockfile

# Build frontend with environment variables
COPY web/ ./
ARG VITE_OG_R2_BASE_URL
RUN bun run build

# ========== Stage 5: Final Rust Build (with embedded assets) ==========
FROM chef AS final-builder

# Cook dependencies (cached from earlier)
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Copy SQLx offline cache and migrations for compile-time macros
COPY .sqlx/ ./.sqlx/
COPY migrations/ ./migrations/

# Copy frontend assets for embedding
COPY --from=frontend /build/build/client ./web/build/client
COPY --from=frontend /build/build/prerendered ./web/build/prerendered

# Build with real assets (use sqlx offline mode)
ENV SQLX_OFFLINE=true
RUN cargo build --release && \
    strip target/release/api

# ========== Stage 6: Runtime ==========
FROM oven/bun:1-alpine AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache ca-certificates tzdata

# Copy Rust binary
COPY --from=final-builder /build/target/release/api ./api

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
ENV PORT=8080 \
    LOG_LEVEL=info \
    LOG_JSON=true \
    TZ=Etc/UTC

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD wget -q --spider http://localhost:${PORT}/api/health || exit 1

ENTRYPOINT ["bun", "run", "/app/web/entrypoint.ts"]
