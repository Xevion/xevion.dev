import { env, waitUntil } from "cloudflare:workers";
import { eq } from "drizzle-orm";
import { render } from "svelte/server";
import type { Db } from "./db/index.ts";
import { projects } from "./db/schema.ts";
import { renderPng } from "./og-render.ts";
import OgImage from "#lib/components/OgImage.svelte";
import { getAppLogger } from "#lib/logger.ts";
import { loadOGFonts } from "#lib/og-fonts.ts";
import type { OGImageSpec } from "#lib/og-types.ts";

const logger = getAppLogger("og");

export function ogKey(spec: OGImageSpec): string {
  switch (spec.type) {
    case "index":
      return "og/index.png";
    case "projects":
      return "og/projects.png";
    case "project":
      return `og/project/${spec.id}.png`;
  }
}

// Project cards are fetched with a `?v=<updatedAt>` bust, so they can be pinned. The shared cards
// have a fixed URL and must expire.
export function ogCacheControl(spec: OGImageSpec): string {
  return spec.type === "project"
    ? "public, max-age=31536000, immutable"
    : "public, max-age=86400";
}

interface CardContent {
  title: string;
  subtitle?: string;
}

async function cardContent(db: Db, spec: OGImageSpec): Promise<CardContent> {
  switch (spec.type) {
    case "index":
      return {
        title: "Ryan Walters",
        subtitle: "Full-Stack Software Engineer",
      };
    case "projects":
      return {
        title: "Projects",
        subtitle: "created, maintained, or contributed to by me...",
      };
    case "project": {
      const [project] = await db
        .select({
          name: projects.name,
          shortDescription: projects.shortDescription,
          hidden: projects.hidden,
        })
        .from(projects)
        .where(eq(projects.id, spec.id))
        .limit(1);
      if (!project || project.hidden) {
        return { title: "Project", subtitle: "View on xevion.dev" };
      }
      return {
        title: project.name,
        subtitle: project.shortDescription || undefined,
      };
    }
  }
}

/** Renders a card to PNG bytes without touching R2. */
export async function renderOgCard(
  db: Db,
  spec: OGImageSpec,
): Promise<Uint8Array> {
  const [content, fonts] = await Promise.all([
    cardContent(db, spec),
    loadOGFonts(),
  ]);
  const { body } = render(OgImage, {
    props: { ...content, type: spec.type },
  });
  return renderPng(body, fonts);
}

async function generateOgImage(db: Db, spec: OGImageSpec): Promise<void> {
  const key = ogKey(spec);
  const png = await renderOgCard(db, spec);
  await env.BUCKET.put(key, png, {
    httpMetadata: {
      contentType: "image/png",
      cacheControl: ogCacheControl(spec),
    },
  });
  logger.info("og image stored", { key, bytes: png.byteLength });
}

/** Regenerates one OG card in the background (waitUntil); never throws. */
export function scheduleOgImage(db: Db, spec: OGImageSpec): void {
  waitUntil(
    generateOgImage(db, spec).catch((cause: unknown) => {
      logger.warn("og image generation failed", {
        key: ogKey(spec),
        cause: cause instanceof Error ? cause.stack : String(cause),
      });
    }),
  );
}

/** Deletes a project's OG card in the background (waitUntil); never throws. */
export function scheduleOgDelete(projectId: string): void {
  const key = ogKey({ type: "project", id: projectId });
  waitUntil(
    env.BUCKET.delete(key).catch((cause: unknown) => {
      logger.warn("og image delete failed", { key, cause: String(cause) });
    }),
  );
}

/** Generates every OG card that is missing from R2. Used by the daily cron. */
export async function ensureOgImages(db: Db): Promise<void> {
  const rows = await db.select({ id: projects.id }).from(projects);
  const specs: OGImageSpec[] = [
    { type: "index" },
    { type: "projects" },
    ...rows.map((r): OGImageSpec => ({ type: "project", id: r.id })),
  ];

  let generated = 0;
  let failed = 0;
  for (const spec of specs) {
    const key = ogKey(spec);
    try {
      if (await env.BUCKET.head(key)) continue;
      await generateOgImage(db, spec);
      generated++;
    } catch (cause) {
      failed++;
      logger.error("og backfill failed", {
        key,
        cause: cause instanceof Error ? cause.stack : String(cause),
      });
    }
  }
  logger.info("og backfill finished", {
    total: specs.length,
    generated,
    failed,
  });
}
