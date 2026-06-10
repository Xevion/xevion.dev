import { renderToHTMLString } from "@tiptap/static-renderer/pm/html-string";
import sanitizeHtml from "sanitize-html";
import type { JSONContent } from "@tiptap/core";
import { tiptapExtensions } from "./extensions";

/**
 * Allowlist for the rendered detail HTML. The content is already constrained by
 * the ProseMirror schema (no raw-HTML passthrough), so this is defense-in-depth
 * — it bounds what a future content-import path or schema change could emit.
 */
const sanitizeOptions: sanitizeHtml.IOptions = {
  allowedTags: sanitizeHtml.defaults.allowedTags.concat([
    "h1",
    "h2",
    "img",
    "figure",
    "figcaption",
    "s",
  ]),
  allowedAttributes: {
    ...sanitizeHtml.defaults.allowedAttributes,
    a: ["href", "name", "target", "rel"],
    "*": ["class"],
  },
  allowedSchemes: ["http", "https", "mailto"],
};

/** Render TipTap/ProseMirror document JSON to sanitized HTML. Server-only. */
export function renderDetailContent(content: JSONContent): string {
  const html = renderToHTMLString({ extensions: tiptapExtensions, content });
  return sanitizeHtml(html, sanitizeOptions);
}
