import { env, waitUntil } from "cloudflare:workers";
import type { ApiSiteIdentity } from "#lib/bindings/index.ts";
import { getAppLogger } from "#lib/logger.ts";
import { apiError } from "./api.ts";
import type { Db } from "./db/index.ts";
import { getResumeKey, setResumeKey } from "./settings.ts";

const logger = getAppLogger("resume");

const MAX_RESUME_BYTES = 10 * 1024 * 1024;
const PDF_MAGIC = [0x25, 0x50, 0x44, 0x46];

/**
 * Stores the PDF under a fresh key so cached URLs of the old one stay valid, then repoints the
 * site at it and removes the old object on a best-effort basis.
 */
export async function uploadResume(
  db: Db,
  file: File,
): Promise<ApiSiteIdentity> {
  if (file.type !== "application/pdf") {
    apiError(
      400,
      "VALIDATION_ERROR",
      `Content type '${file.type || "unknown"}' is not supported. Only application/pdf is accepted`,
    );
  }
  if (file.size > MAX_RESUME_BYTES)
    apiError(400, "VALIDATION_ERROR", "Resume must be 10 MB or smaller");

  const bytes = await file.arrayBuffer();
  const head = new Uint8Array(bytes, 0, Math.min(4, bytes.byteLength));
  if (!PDF_MAGIC.every((b, i) => head[i] === b))
    apiError(400, "VALIDATION_ERROR", "File is not a valid PDF");

  const key = `resume/${crypto.randomUUID()}.pdf`;
  await env.BUCKET.put(key, bytes, {
    httpMetadata: {
      contentType: "application/pdf",
      cacheControl: "public, max-age=31536000, immutable",
    },
  });

  const oldKey = await getResumeKey(db);
  let identity: ApiSiteIdentity;
  try {
    identity = await setResumeKey(db, key, env.MEDIA_BASE_URL);
  } catch (err) {
    waitUntil(env.BUCKET.delete(key));
    throw err;
  }

  if (oldKey && oldKey !== key) {
    waitUntil(
      env.BUCKET.delete(oldKey).catch((cause: unknown) => {
        logger.warn("old resume left in bucket", {
          oldKey,
          cause: String(cause),
        });
      }),
    );
  }
  logger.info("resume uploaded", { key, bytes: bytes.byteLength });
  return identity;
}
