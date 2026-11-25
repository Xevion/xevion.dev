import { env } from "cloudflare:workers";
import { and, asc, eq, inArray, sql } from "drizzle-orm";
import type { Db } from "./db/index.ts";
import { projectMedia, projects } from "./db/schema.ts";
import { apiError } from "./api.ts";
import {
  classifyMime,
  extensionForMime,
  isTooSmall,
  isValidBlurhash,
  MAX_IMAGE_BYTES,
  MAX_VIDEO_BYTES,
  planVariants,
  VARIANT_WIDTHS,
  type VariantName,
} from "./media-plan.ts";
import {
  toApiVariants,
  type StoredImageVariant,
  type StoredVariants,
} from "./media-variants.ts";
import type { ApiProjectMedia } from "#lib/bindings/index.ts";
import { getAppLogger } from "#lib/logger.ts";

const logger = getAppLogger("media");

const IMMUTABLE = "public, max-age=31536000, immutable";
// D1 allows 100 bound parameters per statement.
const IN_CHUNK = 90;
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

type MediaRow = typeof projectMedia.$inferSelect;

function toApiMedia(row: MediaRow): ApiProjectMedia {
  return {
    id: row.id,
    displayOrder: row.displayOrder,
    mediaType: row.mediaType,
    variants: toApiVariants(
      env.MEDIA_BASE_URL,
      row.r2BasePath,
      row.mediaType,
      row.variants,
    ),
    blurhash: row.blurhash ?? undefined,
    metadata: row.metadata ?? undefined,
  };
}

/** Resolves a UUID-or-slug reference to the project's id and visibility. */
export async function findProjectByRef(
  db: Db,
  ref: string,
): Promise<{ id: string; hidden: boolean } | null> {
  const [row] = await db
    .select({ id: projects.id, hidden: projects.hidden })
    .from(projects)
    .where(
      UUID.test(ref)
        ? eq(projects.id, ref.toLowerCase())
        : eq(projects.slug, ref),
    )
    .limit(1);
  return row ?? null;
}

/** Media for many projects at once, keyed by project id, each list in display order. */
export async function listMediaForProjects(
  db: Db,
  projectIds: string[],
): Promise<Map<string, ApiProjectMedia[]>> {
  const result = new Map<string, ApiProjectMedia[]>(
    projectIds.map((id) => [id, []]),
  );
  for (let i = 0; i < projectIds.length; i += IN_CHUNK) {
    const rows = await db
      .select()
      .from(projectMedia)
      .where(inArray(projectMedia.projectId, projectIds.slice(i, i + IN_CHUNK)))
      .orderBy(asc(projectMedia.projectId), asc(projectMedia.displayOrder));
    for (const row of rows) result.get(row.projectId)?.push(toApiMedia(row));
  }
  return result;
}

export async function listProjectMedia(
  db: Db,
  projectId: string,
): Promise<ApiProjectMedia[]> {
  return (await listMediaForProjects(db, [projectId])).get(projectId) ?? [];
}

const streamOf = (bytes: ArrayBuffer) => new Blob([bytes]).stream();

interface ProcessedImage {
  mime: string;
  original: StoredImageVariant;
  variants: Record<
    VariantName,
    { stored: StoredImageVariant; bytes: ArrayBuffer }
  >;
}

async function processImage(bytes: ArrayBuffer): Promise<ProcessedImage> {
  let info: ImageInfoResponse;
  try {
    info = await env.IMAGES.info(streamOf(bytes));
  } catch (cause) {
    logger.warn("image info failed", { cause: String(cause) });
    return apiError(400, "VALIDATION_ERROR", "File is not a readable image");
  }
  if (!("width" in info) || classifyMime(info.format) !== "image") {
    return apiError(
      400,
      "VALIDATION_ERROR",
      `Image format '${info.format}' is not supported. Supported: JPEG, PNG, GIF, WebP, AVIF`,
    );
  }
  if (isTooSmall(info.width, info.height)) {
    return apiError(
      400,
      "VALIDATION_ERROR",
      `Image too small: both sides must be at least ${VARIANT_WIDTHS.thumb}px`,
    );
  }

  const ext = extensionForMime(info.format);
  try {
    const entries = await Promise.all(
      planVariants(info.width, info.height).map(async (plan) => {
        const out = await env.IMAGES.input(streamOf(bytes))
          .transform({ width: plan.width, fit: "scale-down" })
          .output({ format: "image/webp", anim: false });
        return [
          plan.name,
          {
            stored: {
              key: `${plan.name}.webp`,
              width: plan.width,
              height: plan.height,
            },
            bytes: await new Response(out.image()).arrayBuffer(),
          },
        ] as const;
      }),
    );
    return {
      mime: info.format,
      original: {
        key: `original.${ext}`,
        width: info.width,
        height: info.height,
      },
      variants: Object.fromEntries(entries) as ProcessedImage["variants"],
    };
  } catch (cause) {
    logger.warn("image transform failed", { cause: String(cause) });
    return apiError(400, "VALIDATION_ERROR", "Failed to process image");
  }
}

export interface MediaUpload {
  file: File;
  blurhash?: string | null;
}

/** Stores an uploaded image or video and appends it to the project's media. */
export async function addProjectMedia(
  db: Db,
  projectId: string,
  upload: MediaUpload,
): Promise<ApiProjectMedia> {
  const { file } = upload;
  const kind = classifyMime(file.type);
  if (!kind) {
    return apiError(
      400,
      "VALIDATION_ERROR",
      `Content type '${file.type || "unknown"}' is not supported. Supported: JPEG, PNG, GIF, WebP, AVIF, MP4, WebM, MOV`,
    );
  }
  const limit = kind === "image" ? MAX_IMAGE_BYTES : MAX_VIDEO_BYTES;
  if (file.size > limit) {
    return apiError(
      400,
      "VALIDATION_ERROR",
      `File is too large (limit ${Math.floor(limit / 1024 / 1024)} MB)`,
    );
  }
  const blurhash = upload.blurhash?.trim() || null;
  if (blurhash && kind === "image" && !isValidBlurhash(blurhash)) {
    return apiError(400, "VALIDATION_ERROR", "Invalid blurhash", {
      blurhash: "Not a valid blurhash string",
    });
  }

  const basePath = `projects/${projectId}/${crypto.randomUUID()}`;
  const put = (key: string, body: ArrayBuffer | ReadableStream, mime: string) =>
    env.BUCKET.put(`${basePath}/${key}`, body, {
      httpMetadata: { contentType: mime, cacheControl: IMMUTABLE },
    });

  let variants: StoredVariants;
  const written: string[] = [];
  try {
    if (kind === "image") {
      const bytes = await file.arrayBuffer();
      const processed = await processImage(bytes);
      variants = {
        original: processed.original,
        thumb: processed.variants.thumb.stored,
        medium: processed.variants.medium.stored,
        full: processed.variants.full.stored,
      };
      const puts: [string, ArrayBuffer, string][] = [
        [processed.original.key, bytes, processed.mime],
        ...(["thumb", "medium", "full"] as const).map(
          (n): [string, ArrayBuffer, string] => [
            processed.variants[n].stored.key,
            processed.variants[n].bytes,
            "image/webp",
          ],
        ),
      ];
      await Promise.all(
        puts.map(async ([key, body, mime]) => {
          written.push(key);
          await put(key, body, mime);
        }),
      );
    } else {
      const key = `original.${extensionForMime(file.type)}`;
      variants = { original: { key, mime: file.type } };
      written.push(key);
      await put(key, file.stream(), file.type);
    }

    const [row] = await db
      .insert(projectMedia)
      .values({
        projectId,
        // One statement, so concurrent uploads each claim a distinct slot.
        displayOrder: sql`(select coalesce(max(display_order) + 1, 0) from project_media where project_id = ${projectId})`,
        mediaType: kind,
        r2BasePath: basePath,
        variants,
        blurhash: kind === "image" ? blurhash : null,
      })
      .returning();
    logger.info("media uploaded", {
      mediaId: row.id,
      projectId,
      kind,
      filename: file.name,
      bytes: file.size,
    });
    return toApiMedia(row);
  } catch (cause) {
    if (written.length > 0) {
      await env.BUCKET.delete(written.map((k) => `${basePath}/${k}`)).catch(
        (e: unknown) => {
          logger.warn("orphaned media objects after failed upload", {
            basePath,
            cause: String(e),
          });
        },
      );
    }
    throw cause;
  }
}

/** Renumbers a project's media. Listed ids come first in the given order, the rest follow. */
export async function reorderProjectMedia(
  db: Db,
  projectId: string,
  mediaIds: string[],
): Promise<ApiProjectMedia[]> {
  if (new Set(mediaIds).size !== mediaIds.length) {
    apiError(400, "VALIDATION_ERROR", "Duplicate media ids", {
      mediaIds: "Media ids must be unique",
    });
  }
  const current = await db
    .select({ id: projectMedia.id })
    .from(projectMedia)
    .where(eq(projectMedia.projectId, projectId))
    .orderBy(asc(projectMedia.displayOrder));
  const known = new Set(current.map((r) => r.id));
  if (mediaIds.some((id) => !known.has(id))) {
    apiError(400, "VALIDATION_ERROR", "Unknown media id", {
      mediaIds: "All media ids must belong to this project",
    });
  }
  const listed = new Set(mediaIds);
  const finalOrder = [
    ...mediaIds,
    ...current.map((r) => r.id).filter((id) => !listed.has(id)),
  ];

  // (project_id, display_order) is unique: park every row on a negative slot first so the
  // final renumbering never collides mid-batch.
  const park = db
    .update(projectMedia)
    .set({ displayOrder: sql`-(${projectMedia.displayOrder}) - 1` })
    .where(eq(projectMedia.projectId, projectId));
  const place = finalOrder.map((id, index) =>
    db
      .update(projectMedia)
      .set({ displayOrder: index })
      .where(
        and(eq(projectMedia.id, id), eq(projectMedia.projectId, projectId)),
      ),
  );
  await db.batch([park, ...place]);

  return listProjectMedia(db, projectId);
}

/** Lists then deletes every object under a prefix. Returns the number deleted. */
async function deletePrefix(prefix: string): Promise<number> {
  let deleted = 0;
  let cursor: string | undefined;
  do {
    const page = await env.BUCKET.list({ prefix, cursor, limit: 1000 });
    if (page.objects.length > 0) {
      await env.BUCKET.delete(page.objects.map((o) => o.key));
      deleted += page.objects.length;
    }
    cursor = page.truncated ? page.cursor : undefined;
  } while (cursor);
  return deleted;
}

/** Deletes one media row after removing its R2 objects. Null when it is not in this project. */
export async function deleteProjectMedia(
  db: Db,
  projectId: string,
  mediaId: string,
): Promise<ApiProjectMedia | null> {
  const [row] = await db
    .select()
    .from(projectMedia)
    .where(
      and(eq(projectMedia.id, mediaId), eq(projectMedia.projectId, projectId)),
    )
    .limit(1);
  if (!row) return null;

  const prefix = `${row.r2BasePath.replace(/\/+$/, "")}/`;
  try {
    const count = await deletePrefix(prefix);
    logger.info("deleted media objects", { mediaId, prefix, count });
  } catch (cause) {
    logger.warn("media objects will be orphaned", {
      mediaId,
      prefix,
      cause: String(cause),
    });
  }

  await db.delete(projectMedia).where(eq(projectMedia.id, mediaId));
  return toApiMedia(row);
}

/** Removes every R2 object under a project's media prefix. Best-effort; logs failures. */
export async function deleteProjectMediaObjects(
  projectId: string,
): Promise<void> {
  const prefix = `projects/${projectId}/`;
  try {
    const count = await deletePrefix(prefix);
    logger.info("deleted project media objects", { projectId, count });
  } catch (cause) {
    logger.warn("project media objects will be orphaned", {
      projectId,
      prefix,
      cause: String(cause),
    });
  }
}
