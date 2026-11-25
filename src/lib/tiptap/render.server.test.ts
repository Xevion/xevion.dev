import { describe, it, expect } from "vitest";
import type { JSONContent } from "@tiptap/core";
import { renderDetailContent } from "./render.server";

/** Wrap inline nodes in a single-paragraph document. */
function paragraph(...inline: JSONContent[]): JSONContent {
  return { type: "doc", content: [{ type: "paragraph", content: inline }] };
}

describe("renderDetailContent inline code", () => {
  it("highlights lang-mode inline code as an inline code block", async () => {
    const { html } = await renderDetailContent(
      paragraph(
        { type: "text", text: "run " },
        {
          type: "text",
          text: "let x = 1",
          marks: [{ type: "code", attrs: { lang: "typescript" } }],
        },
      ),
    );
    // Wrapped, dual-theme token spans, but no block <pre> — an inline code block.
    expect(html).toContain('<code class="rd-inline-code">');
    expect(html).toContain("--shiki-dark");
    expect(html).not.toContain("<pre");
  });

  it("colors token-mode inline code with a semantic class", async () => {
    const { html } = await renderDetailContent(
      paragraph({
        type: "text",
        text: "Arc",
        marks: [{ type: "code", attrs: { token: "type" } }],
      }),
    );
    // Author-declared token: a class, not a grammar pass (no --shiki-dark).
    expect(html).toContain('<span class="tk-type">Arc</span>');
    expect(html).not.toContain("--shiki-dark");
  });

  it("leaves plain inline code untouched", async () => {
    const { html } = await renderDetailContent(
      paragraph({ type: "text", text: "plain", marks: [{ type: "code" }] }),
    );
    expect(html).toContain("<code>plain</code>");
    expect(html).not.toContain("rd-inline-code");
  });

  it("preserves a link wrapping inline code instead of dropping it", async () => {
    const { html } = await renderDetailContent(
      paragraph({
        type: "text",
        text: "linked",
        marks: [
          { type: "link", attrs: { href: "https://example.com" } },
          { type: "code", attrs: { lang: "typescript" } },
        ],
      }),
    );
    expect(html).toContain('href="https://example.com"');
    expect(html).toContain("linked");
  });

  it("degrades an unknown lang to plain-text highlighting without throwing", async () => {
    const { html } = await renderDetailContent(
      paragraph({
        type: "text",
        text: "fizzbuzz",
        marks: [{ type: "code", attrs: { lang: "klingon" } }],
      }),
    );
    // Unknown grammar → "text"; still an inline code block, just unhighlighted.
    expect(html).toContain('<code class="rd-inline-code">');
    expect(html).toContain("fizzbuzz");
  });

  it("renders an unknown token kind as plain inline code, not a token span", async () => {
    const { html } = await renderDetailContent(
      paragraph({
        type: "text",
        text: "Arc",
        marks: [{ type: "code", attrs: { token: "bogus" } }],
      }),
    );
    expect(html).toContain("<code>Arc</code>");
    expect(html).not.toContain("tk-bogus");
    expect(html).not.toContain("rd-inline-code");
  });

  it("prefers token over lang when a span carries both attrs", async () => {
    const { html } = await renderDetailContent(
      paragraph({
        type: "text",
        text: "Arc",
        marks: [{ type: "code", attrs: { lang: "rust", token: "type" } }],
      }),
    );
    // token is checked first: a semantic class, not a grammar pass.
    expect(html).toContain('<span class="tk-type">Arc</span>');
    expect(html).not.toContain("--shiki-dark");
  });

  it("highlights code containing '<' instead of mistaking it for a nested mark", async () => {
    const { html } = await renderDetailContent(
      paragraph({
        type: "text",
        text: "a < b",
        marks: [{ type: "code", attrs: { lang: "rust" } }],
      }),
    );
    // Took the highlight branch (rd-inline-code), not the nested-mark fallback,
    // and the '<' is escaped.
    expect(html).toContain('<code class="rd-inline-code">');
    expect(html).toContain("&lt;");
  });

  it("falls back to plain inline code for an absurdly long lang span", async () => {
    const huge = "x".repeat(5000);
    const { html } = await renderDetailContent(
      paragraph({
        type: "text",
        text: huge,
        marks: [{ type: "code", attrs: { lang: "rust" } }],
      }),
    );
    // Over the highlight cap: escaped <code>, no Shiki pass (the renderer's own
    // guard, independent of the model's write-path length bound).
    expect(html).not.toContain("rd-inline-code");
    expect(html).not.toContain("--shiki-dark");
    expect(html).toContain(huge);
  });

  it("ignores a non-string lang attr instead of treating it as a grammar", async () => {
    const { html } = await renderDetailContent(
      paragraph({
        type: "text",
        text: "plain",
        marks: [{ type: "code", attrs: { lang: 123 } }],
      }),
    );
    expect(html).toContain("<code>plain</code>");
    expect(html).not.toContain("rd-inline-code");
  });
});
