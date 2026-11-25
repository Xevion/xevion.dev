import type { ApiMediaVariants } from "./ApiMediaVariants";
import type { MediaMetadata } from "./MediaMetadata";
import type { MediaType } from "./MediaType";

/**
 * API response type for project media
 */
export type ApiProjectMedia = {
  id: string;
  displayOrder: number;
  mediaType: MediaType;
  variants: ApiMediaVariants;
  blurhash?: string;
  metadata?: MediaMetadata;
};
