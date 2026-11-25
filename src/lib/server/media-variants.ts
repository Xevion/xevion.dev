import type { ApiMediaVariants, MediaType } from "#lib/bindings/index.ts";

export interface StoredImageVariant {
  key: string;
  width: number;
  height: number;
}

export interface StoredVideoOriginal {
  key: string;
  mime: string;
  duration?: number;
}

/** The `variants` JSON column. Keys are relative to the row's `r2_base_path`. */
export interface StoredVariants {
  thumb?: StoredImageVariant;
  medium?: StoredImageVariant;
  full?: StoredImageVariant;
  poster?: StoredImageVariant;
  original?: StoredImageVariant | StoredVideoOriginal;
}

export function toApiVariants(
  mediaBaseUrl: string,
  basePath: string,
  mediaType: MediaType,
  stored: StoredVariants,
): ApiMediaVariants {
  const base = `${mediaBaseUrl}/${basePath.replace(/\/+$/, "")}`;
  const image = (v: StoredImageVariant | undefined) =>
    v && { url: `${base}/${v.key}`, width: v.width, height: v.height };

  const out: ApiMediaVariants = {
    thumb: image(stored.thumb),
    medium: image(stored.medium),
    full: image(stored.full),
    poster: image(stored.poster),
  };
  if (stored.original) {
    if (mediaType === "video") {
      const v = stored.original as StoredVideoOriginal;
      out.video = {
        url: `${base}/${v.key}`,
        mime: v.mime,
        duration: v.duration,
      };
    } else {
      out.original = image(stored.original as StoredImageVariant);
    }
  }
  return out;
}
