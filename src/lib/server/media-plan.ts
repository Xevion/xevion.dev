export const IMAGE_MIMES = [
  "image/jpeg",
  "image/png",
  "image/gif",
  "image/webp",
  "image/avif",
] as const;

export const VIDEO_MIMES = [
  "video/mp4",
  "video/webm",
  "video/quicktime",
] as const;

export const VARIANT_WIDTHS = { thumb: 300, medium: 800, full: 1600 } as const;
export type VariantName = keyof typeof VARIANT_WIDTHS;

/** Images whose smaller side is under this are rejected, so every variant has real detail. */
export const MIN_IMAGE_DIMENSION = VARIANT_WIDTHS.thumb;

export const MAX_IMAGE_BYTES = 30 * 1024 * 1024;
// formData() buffers the whole body, which has to fit in the 128 MB isolate with the R2 copy.
export const MAX_VIDEO_BYTES = 48 * 1024 * 1024;

export function classifyMime(mime: string): "image" | "video" | null {
  if ((IMAGE_MIMES as readonly string[]).includes(mime)) return "image";
  if ((VIDEO_MIMES as readonly string[]).includes(mime)) return "video";
  return null;
}

const EXTENSIONS: Record<string, string> = {
  "image/jpeg": "jpg",
  "image/png": "png",
  "image/gif": "gif",
  "image/webp": "webp",
  "image/avif": "avif",
  "video/mp4": "mp4",
  "video/webm": "webm",
  "video/quicktime": "mov",
};

export function extensionForMime(mime: string): string {
  return EXTENSIONS[mime] ?? "bin";
}

export interface PlannedVariant {
  name: VariantName;
  width: number;
  height: number;
}

/** Target dimensions for each variant: widths are capped at the source width, never enlarged. */
export function planVariants(width: number, height: number): PlannedVariant[] {
  return (Object.keys(VARIANT_WIDTHS) as VariantName[]).map((name) => {
    const target = Math.min(VARIANT_WIDTHS[name], width);
    return {
      name,
      width: target,
      height: Math.round((height * target) / width),
    };
  });
}

export function isTooSmall(width: number, height: number): boolean {
  return Math.min(width, height) < MIN_IMAGE_DIMENSION;
}

// Base83 alphabet used by blurhash.
const BLURHASH = /^[0-9A-Za-z#$%*+,\-.:;=?@[\]^_{|}~]{6,128}$/;

export function isValidBlurhash(value: string): boolean {
  return BLURHASH.test(value);
}
