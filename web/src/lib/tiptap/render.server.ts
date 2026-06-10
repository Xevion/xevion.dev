import { renderToHTMLString } from "@tiptap/static-renderer/pm/html-string";
import sanitizeHtml from "sanitize-html";
import type { JSONContent } from "@tiptap/core";
import { tiptapExtensions } from "./extensions";
import { getHighlighter } from "./shiki.server";

/**
 * Allowlist for the rendered detail HTML. The content is already constrained by
 * the ProseMirror schema (no raw-HTML passthrough), so this is defense-in-depth
 * — it bounds what a future content-import path or schema change could emit.
 *
 * Tags are sanitize-html's safe defaults minus `h1` (the page already owns the
 * sole h1; body headings are h2–h4 per the editor schema). `img` is intentionally
 * absent from the defaults — there is no image node in the schema yet, so nothing
 * should emit one.
 *
 * `style` is allowed only on the code-block elements, and `allowedStyles` is
 * locked to the two declarations Shiki actually emits: a per-token `color` and a
 * `--shiki-dark` custom property (dark mode). That keeps fetch-capable CSS — e.g.
 * `background-image: url(...)` — from riding along; sanitize-html preserves the
 * custom property while dropping everything else.
 */
const sanitizeOptions: sanitizeHtml.IOptions = {
  allowedTags: sanitizeHtml.defaults.allowedTags.filter((tag) => tag !== "h1"),
  allowedAttributes: {
    ...sanitizeHtml.defaults.allowedAttributes,
    a: ["href", "name", "target", "rel"],
    "*": ["class"],
    span: ["style"],
    code: ["style"],
    pre: ["style", "tabindex"],
  },
  allowedStyles: {
    "*": { color: [/.*/], "--shiki-dark": [/.*/] },
  },
  allowedSchemes: ["http", "https", "mailto"],
};

/** Render TipTap/ProseMirror document JSON to sanitized HTML. Server-only. */
export async function renderDetailContent(
  content: JSONContent,
): Promise<string> {
  const highlighter = await getHighlighter();
  const loadedLangs = new Set(highlighter.getLoadedLanguages());

  const html = renderToHTMLString({
    extensions: tiptapExtensions,
    content,
    options: {
      nodeMapping: {
        codeBlock: ({ node }) => {
          const requested = (node.attrs.language as string | null) ?? "text";
          const lang = loadedLangs.has(requested) ? requested : "text";
          // codeToHtml is sync once the highlighter has resolved. Dual themes:
          // light is emitted as inline color, dark as a --shiki-dark var that a
          // `.dark` ancestor rule activates on the public page.
          return highlighter.codeToHtml(node.textContent, {
            lang,
            themes: { light: "github-light", dark: "github-dark" },
          });
        },
      },
    },
  });

  return sanitizeHtml(html, sanitizeOptions);
}
