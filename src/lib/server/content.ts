import { apiError } from "#lib/server/api.ts";
import { renderDetailContent } from "#lib/tiptap/render.server.ts";
import type { TocItem } from "#lib/tiptap/toc.ts";
import { prepareDoc } from "#lib/tiptap/validate.ts";

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
  if (doc === null || doc === undefined)
    return { detailContent: null, renderedHtml: null, toc: null };

  const prepared = prepareDoc(doc);
  if (!prepared.ok) apiError(400, "VALIDATION_ERROR", prepared.message);
  if (prepared.doc === null)
    return { detailContent: null, renderedHtml: null, toc: null };

  const { html, toc } = await renderDetailContent(prepared.doc);
  return { detailContent: prepared.doc, renderedHtml: html, toc };
}
