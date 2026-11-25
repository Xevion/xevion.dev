import type { ApiMediaVariant } from "./ApiMediaVariant";
import type { ApiVideoOriginal } from "./ApiVideoOriginal";

/**
 * API response for media variants
 */
export type ApiMediaVariants = { thumb?: ApiMediaVariant, medium?: ApiMediaVariant, full?: ApiMediaVariant, original?: ApiMediaVariant, poster?: ApiMediaVariant, video?: ApiVideoOriginal, };
