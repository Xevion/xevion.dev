FROM oven/bun:1-alpine AS base
WORKDIR /app
RUN apk add --no-cache libc6-compat

# Install dependencies
FROM base AS deps
COPY package.json bun.lock ./
COPY packages/db/package.json ./packages/db/
COPY packages/types/package.json ./packages/types/
COPY packages/typescript-config/package.json ./packages/typescript-config/
COPY apps/payload/package.json ./apps/payload/
COPY apps/web/package.json ./apps/web/
RUN bun install --frozen-lockfile

# Build everything
FROM base AS builder
COPY --from=deps /app ./
COPY . .

ENV NEXT_TELEMETRY_DISABLED=1
RUN bun run build

# Compile SvelteKit to standalone executable
RUN bun build --compile --target=bun --minify --outfile=apps/web/web-server ./apps/web/build/index.js

# Production image
FROM oven/bun:1-alpine AS runner
WORKDIR /app

RUN apk add --no-cache caddy && \
    addgroup --system --gid 1001 nodejs && \
    adduser --system --uid 1001 nextjs

ENV NODE_ENV=production
ENV PORT=3000
ENV NEXT_TELEMETRY_DISABLED=1

# Copy built artifacts
COPY --from=builder --chown=nextjs:nodejs /app/packages/db/dist ./packages/db/dist
COPY --from=builder --chown=nextjs:nodejs /app/packages/db/package.json ./packages/db/
COPY --from=builder --chown=nextjs:nodejs /app/packages/types/dist ./packages/types/dist
COPY --from=builder --chown=nextjs:nodejs /app/packages/types/package.json ./packages/types/

# Payload standalone build
COPY --from=builder --chown=nextjs:nodejs /app/apps/payload/.next/standalone ./apps/payload/.next/standalone
COPY --from=builder --chown=nextjs:nodejs /app/apps/payload/.next/static ./apps/payload/.next/standalone/apps/payload/.next/static
COPY --from=builder --chown=nextjs:nodejs /app/apps/payload/public ./apps/payload/.next/standalone/apps/payload/public
COPY --from=builder --chown=nextjs:nodejs /app/apps/payload/src/migrations ./apps/payload/.next/standalone/apps/payload/src/migrations

# Web standalone executable (no node_modules needed)
COPY --from=builder --chown=nextjs:nodejs /app/apps/web/web-server ./apps/web/
COPY --from=builder --chown=nextjs:nodejs /app/apps/web/build/client ./apps/web/build/client

# Entrypoint
COPY --chown=nextjs:nodejs Caddyfile docker-entrypoint.ts ./
RUN chmod +x docker-entrypoint.ts && \
    caddy fmt --overwrite Caddyfile

USER nextjs
EXPOSE ${PORT}
ENTRYPOINT ["bun", "run", "docker-entrypoint.ts"]
