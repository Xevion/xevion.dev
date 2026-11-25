import { describe, expect, it } from "vitest";
import { prepareContent } from "#lib/server/content.ts";

describe("prepareContent under workerd", () => {
  it("validates, stamps ids and renders highlighted code", async () => {
    const prepared = await prepareContent({
      type: "doc",
      content: [
        {
          type: "heading",
          attrs: { level: 2 },
          content: [{ type: "text", text: "Setup" }],
        },
        {
          type: "codeBlock",
          attrs: { language: "rust" },
          content: [{ type: "text", text: "fn main() {}" }],
        },
        {
          type: "paragraph",
          content: [
            { type: "text", text: "Use " },
            {
              type: "text",
              text: "let x = 1",
              marks: [{ type: "code", attrs: { lang: "typescript" } }],
            },
            { type: "text", text: " and " },
            {
              type: "text",
              text: "Arc",
              marks: [{ type: "code", attrs: { token: "type" } }],
            },
          ],
        },
      ],
    });

    expect(prepared.renderedHtml).toContain('<div class="rd-codeblock">');
    expect(prepared.renderedHtml).toContain("--shiki-dark");
    expect(prepared.renderedHtml).toContain('<code class="rd-inline-code">');
    expect(prepared.renderedHtml).toContain('<span class="tk-type">Arc</span>');
    expect(prepared.toc).toEqual([{ level: 2, text: "Setup", id: "setup" }]);

    const stored = prepared.detailContent as {
      content: { attrs?: { id?: string } }[];
    };
    for (const block of stored.content)
      expect(block.attrs?.id).toMatch(/^[a-z0-9]{8}$/);
  });

  it("stores an empty document as nulls", async () => {
    expect(await prepareContent({ type: "doc" })).toEqual({
      detailContent: null,
      renderedHtml: null,
      toc: null,
    });
  });

  it("rejects an invalid document with a 400", async () => {
    await expect(
      prepareContent({ type: "doc", content: [{ type: "script" }] }),
    ).rejects.toMatchObject({ status: 400 });
  });
});
