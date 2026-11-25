import { MarkdownManager } from "@tiptap/markdown";
import { tiptapExtensions } from "./extensions.ts";
import type { DocNode, Mark } from "./ops.ts";

export class MarkdownError extends Error {
  constructor(
    readonly kind: "empty" | "unsupported",
    message: string,
  ) {
    super(message);
    this.name = "MarkdownError";
  }
}

const MAX_HEADING_LEVEL = 4;

interface Token {
  type: string;
  task?: boolean;
  tokens?: Token[];
  items?: Token[];
}

let manager: MarkdownManager | undefined;

function getManager(): MarkdownManager {
  manager ??= new MarkdownManager({ extensions: tiptapExtensions });
  return manager;
}

/** The first construct in the token tree with no equivalent in the detail schema. */
function findUnsupported(tokens: Token[]): string | undefined {
  for (const token of tokens) {
    switch (token.type) {
      case "image":
        return "images";
      case "table":
        return "tables";
      case "html":
        return "raw HTML";
    }
    if (token.task) return "task lists";
    const nested = findUnsupported([
      ...(token.tokens ?? []),
      ...(token.items ?? []),
    ]);
    if (nested) return nested;
  }
  return undefined;
}

const HINT_VALUE = /^[A-Za-z0-9\-_+#]+$/;

/**
 * Peel a trailing `{:lang}` (grammar) or `{:.kind}` (semantic token) off an
 * inline-code span. A span with no well-formed suffix, or nothing before it,
 * is returned verbatim so literal braces survive.
 */
export function splitCodeHint(code: string): {
  text: string;
  attrs?: Record<string, string>;
} {
  if (!code.endsWith("}")) return { text: code };
  const inner = code.slice(0, -1);
  const open = inner.lastIndexOf("{:");
  if (open < 0) return { text: code };
  const text = code.slice(0, open);
  const spec = inner.slice(open + 2);
  if (text === "") return { text: code };
  if (spec.startsWith(".") && HINT_VALUE.test(spec.slice(1))) {
    return { text, attrs: { token: spec.slice(1) } };
  }
  if (HINT_VALUE.test(spec)) return { text, attrs: { lang: spec } };
  return { text: code };
}

function cleanAttrs(
  attrs: Record<string, unknown> | undefined,
): Record<string, unknown> | undefined {
  if (!attrs) return undefined;
  const kept = Object.fromEntries(
    Object.entries(attrs).filter(([, v]) => v !== null && v !== undefined),
  );
  return Object.keys(kept).length > 0 ? kept : undefined;
}

function normalizeMark(mark: Mark): Mark {
  const attrs = cleanAttrs(mark.attrs);
  return attrs ? { type: mark.type, attrs } : { type: mark.type };
}

function normalizeNode(node: DocNode, inCodeBlock: boolean): DocNode {
  const out: DocNode = { type: node.type };
  let attrs = cleanAttrs(node.attrs);

  if (node.type === "heading" && typeof attrs?.level === "number") {
    attrs = { ...attrs, level: Math.min(attrs.level + 1, MAX_HEADING_LEVEL) };
  }
  if (node.type === "orderedList" && attrs?.start === 1) {
    const rest = { ...attrs };
    delete rest.start;
    attrs = Object.keys(rest).length > 0 ? rest : undefined;
  }

  if (node.type === "text") {
    let text = node.text ?? "";
    let marks = (node.marks ?? []).map(normalizeMark);
    if (!inCodeBlock) {
      text = text.replaceAll("\n", " ");
      const codeIndex = marks.findIndex((m) => m.type === "code");
      if (codeIndex >= 0) {
        const hint = splitCodeHint(text);
        text = hint.text;
        if (hint.attrs) {
          marks = marks.map((m, i) =>
            i === codeIndex
              ? { ...m, attrs: { ...m.attrs, ...hint.attrs } }
              : m,
          );
        }
      }
    }
    out.text = text;
    if (marks.length > 0) out.marks = marks;
    return out;
  }

  if (attrs) out.attrs = attrs;
  if (node.content && node.content.length > 0) {
    const inCode = node.type === "codeBlock";
    out.content = node.content.map((c) => normalizeNode(c, inCode));
  }
  return out;
}

function isBlankParagraph(node: DocNode): boolean {
  return node.type === "paragraph" && (node.content ?? []).length === 0;
}

/**
 * Convert a Markdown snippet into detail-document blocks. Headings shift down
 * one level (`#` becomes h2, deeper levels clamp to h4) so the body sits under
 * the page title; constructs the schema cannot hold are rejected.
 */
export function markdownToBlocks(markdown: string): DocNode[] {
  const md = getManager();
  const unsupported = findUnsupported(md.instance.lexer(markdown) as Token[]);
  if (unsupported) {
    throw new MarkdownError(
      "unsupported",
      `${unsupported} are not supported in detail content`,
    );
  }
  const blocks = (md.parse(markdown).content ?? []) as DocNode[];
  if (blocks.every(isBlankParagraph)) {
    throw new MarkdownError("empty", "the Markdown produced no content");
  }
  return blocks.map((b) => normalizeNode(b, false));
}
