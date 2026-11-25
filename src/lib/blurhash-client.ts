import { encode } from "blurhash";

const SIZE = 32;

/** Blurhash of an image file from a 32x32 downscale, or undefined when it can't be decoded. */
export async function blurhashForImage(
  file: File,
): Promise<string | undefined> {
  try {
    const bitmap = await createImageBitmap(file);
    try {
      const canvas = document.createElement("canvas");
      canvas.width = SIZE;
      canvas.height = SIZE;
      const ctx = canvas.getContext("2d");
      if (!ctx) return undefined;
      ctx.drawImage(bitmap, 0, 0, SIZE, SIZE);
      const { data } = ctx.getImageData(0, 0, SIZE, SIZE);
      return encode(data, SIZE, SIZE, 4, 3);
    } finally {
      bitmap.close();
    }
  } catch {
    return undefined;
  }
}
