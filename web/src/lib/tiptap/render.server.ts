import { renderToHTMLString } from "@tiptap/static-renderer/pm/html-string";
import sanitizeHtml from "sanitize-html";
import GithubSlugger from "github-slugger";
import type { JSONContent } from "@tiptap/core";
import { tiptapExtensions } from "./extensions";
import { getHighlighter } from "./shiki.server";
import { codeTokenIds } from "./code-tokens";

/**
 * Cap on the length of a `lang`-highlighted inline-code run. Inline code is short
 * by nature; beyond this the span renders as plain escaped <code> rather than
 * running through Shiki, so a pathological run can't stall SSR. The model enforces
 * its own (higher) write-path bound — this is the renderer's independent guard for
 * content that might reach it unvalidated.
 */
const MAX_INLINE_HIGHLIGHT_CHARS = 2000;

/** A heading entry for the on-page table of contents (h2/h3). */
export interface TocItem {
  level: number;
  text: string;
  id: string;
}

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
  allowedTags: [
    ...sanitizeHtml.defaults.allowedTags.filter((tag) => tag !== "h1"),
    "kbd",
    "figure",
    "figcaption",
    "img",
    "video",
    "aside",
    "details",
    "summary",
  ],
  allowedAttributes: {
    ...sanitizeHtml.defaults.allowedAttributes,
    a: ["href", "name", "target", "rel"],
    "*": ["class"],
    span: ["style", "data-note", "tabindex"],
    code: ["style"],
    pre: ["style", "tabindex"],
    img: ["src", "alt", "loading"],
    video: ["src", "autoplay", "loop", "muted", "playsinline", "poster"],
    aside: ["data-variant"],
    details: ["open"],
    h2: ["id"],
    h3: ["id"],
    h4: ["id"],
  },
  allowedStyles: {
    "*": { color: [/.*/], "--shiki-dark": [/.*/] },
  },
  allowedSchemes: ["http", "https", "mailto"],
};

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/**
 * Turn `[[Key]]` tokens in the prose into `<kbd>` keycaps, leaving any `<pre>`
 * code blocks untouched so code samples that happen to contain `[[…]]` survive.
 */
function applyKeycaps(html: string): string {
  return html
    .split(/(<pre[\s\S]*?<\/pre>)/g)
    .map((segment, i) =>
      i % 2 === 1
        ? segment
        : segment.replace(
            /\[\[([^\]]+)\]\]/g,
            (_, key: string) => `<kbd>${escapeHtml(key)}</kbd>`,
          ),
    )
    .join("");
}

/**
 * Render TipTap/ProseMirror document JSON to sanitized HTML plus a table of
 * contents. Server-only.
 *
 * Headings get text-derived slug `id`s (deduped by github-slugger, matching the
 * GitHub/remark convention) so they double as shareable anchor targets, and each
 * carries a hover-revealed permalink. The same pass collects the `toc` the page
 * renders into the rail for scroll-spy nav.
 */
export async function renderDetailContent(
  content: JSONContent,
): Promise<{ html: string; toc: TocItem[] }> {
  const highlighter = await getHighlighter();
  const loadedLangs = new Set(highlighter.getLoadedLanguages());

  const slugger = new GithubSlugger();
  const toc: TocItem[] = [];

  const rendered = renderToHTMLString({
    extensions: tiptapExtensions,
    content,
    options: {
      nodeMapping: {
        heading: ({ node, children }) => {
          const level = (node.attrs.level as number | null) ?? 2;
          const text = node.textContent;
          const id = slugger.slug(text);
          const inner = Array.isArray(children)
            ? children.join("")
            : (children ?? "");
          toc.push({ level, text, id });
          // The heading text itself is the permalink (the slug id is the anchor
          // target); it reads as plain heading text and only tints on hover, so
          // it doesn't compete with the §NN section counter.
          return `<h${level} id="${id}" class="rd-heading"><a class="rd-anchor" href="#${id}">${inner}</a></h${level}>`;
        },
        codeBlock: ({ node }) => {
          const requested = (node.attrs.language as string | null) ?? "text";
          const lang = loadedLangs.has(requested) ? requested : "text";
          // codeToHtml is sync once the highlighter has resolved. Dual themes:
          // light is emitted as inline color, dark as a --shiki-dark var that a
          // `.dark` ancestor rule activates on the public page.
          const shiki = highlighter.codeToHtml(node.textContent, {
            lang,
            themes: { light: "github-light", dark: "github-dark" },
          });
          // A labeled header bar carries the language; plain `text` blocks omit it.
          const showLabel =
            requested && requested !== "text" && requested !== "plaintext";
          const head = showLabel
            ? `<div class="rd-codeblock-head">${escapeHtml(requested)}</div>`
            : "";
          return `<div class="rd-codeblock">${head}<div class="rd-codeblock-body">${shiki}</div></div>`;
        },
        figure: ({ node }) => {
          const src = (node.attrs.src as string | null) ?? "";
          if (!src) return "";
          const alt = (node.attrs.alt as string | null) ?? "";
          const caption = (node.attrs.caption as string | null) ?? "";
          const kind = (node.attrs.kind as string | null) ?? "image";
          const media =
            kind === "video"
              ? `<video class="rd-figure-media" src="${escapeHtml(src)}" autoplay loop muted playsinline></video>`
              : `<img class="rd-figure-media" src="${escapeHtml(src)}" alt="${escapeHtml(alt)}" loading="lazy" />`;
          const cap = caption
            ? `<figcaption class="rd-figure-cap">${escapeHtml(caption)}</figcaption>`
            : "";
          return `<figure class="rd-figure">${media}${cap}</figure>`;
        },
      },
      markMapping: {
        // Inline code highlighting. `lang` runs the span through Shiki with
        // `structure: "inline"` — the code-block tokens minus the <pre>/.line
        // wrappers — while `token` paints a single author-declared kind via a
        // `.tk-*` class. A code mark with neither stays a plain <code>.
        code: ({ mark, node, children }) => {
          const inner = Array.isArray(children)
            ? children.join("")
            : (children ?? "");
          // Attrs are author/model data, not guaranteed strings — a non-string
          // lang/token is treated as absent rather than coerced into markup.
          const lang =
            typeof mark.attrs?.lang === "string" ? mark.attrs.lang : null;
          const token =
            typeof mark.attrs?.token === "string" ? mark.attrs.token : null;
          // A nested mark (a link around this code) renders HTML into `inner`;
          // re-tokenizing the raw text would discard it, so only highlight a
          // standalone text run — escaped text never contains a literal "<".
          const nested = inner.includes("<");
          if (nested || (!lang && !token)) {
            return `<code>${inner}</code>`;
          }
          const raw = (node.text ?? node.textContent ?? "") as string;
          if (token && codeTokenIds.has(token)) {
            return `<code class="rd-inline-code"><span class="tk-${token}">${escapeHtml(raw)}</span></code>`;
          }
          if (lang && raw.length <= MAX_INLINE_HIGHLIGHT_CHARS) {
            const grammar = loadedLangs.has(lang) ? lang : "text";
            const spans = highlighter.codeToHtml(raw, {
              lang: grammar,
              themes: { light: "github-light", dark: "github-dark" },
              structure: "inline",
            });
            return `<code class="rd-inline-code">${spans}</code>`;
          }
          return `<code>${inner}</code>`;
        },
      },
    },
  });

  const html = sanitizeHtml(applyKeycaps(rendered), sanitizeOptions);
  return { html, toc };
}
