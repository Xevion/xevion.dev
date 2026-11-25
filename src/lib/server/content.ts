import type { TocItem } from "#lib/tiptap/toc.ts";

export interface PreparedContent {
  /** Canonical stored document, or null for an empty/absent doc. */
  detailContent: unknown | null;
  renderedHtml: string | null;
  toc: TocItem[] | null;
}

/**
 * Validates a detail document against the schema, stamps missing block ids, and renders the
 * public HTML and TOC. Throws apiError(400, ...) when the document is invalid.
 */
export async function prepareContent(doc: unknown): Promise<PreparedContent> {
  void doc;
  throw new Error("prepareContent: not implemented");
}
