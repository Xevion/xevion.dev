import { describe, expect, it } from "vitest";
import { MarkdownError, markdownToBlocks, splitCodeHint } from "./markdown.ts";
import type { DocNode } from "./ops.ts";

const text = (t: string, marks?: DocNode["marks"]): DocNode =>
  marks ? { type: "text", text: t, marks } : { type: "text", text: t };

describe("markdownToBlocks", () => {
  it("converts a paragraph with inline marks", () => {
    expect(markdownToBlocks("Hello **bold** and *italic* and `code`.")).toEqual(
      [
        {
          type: "paragraph",
          content: [
            text("Hello "),
            text("bold", [{ type: "bold" }]),
            text(" and "),
            text("italic", [{ type: "italic" }]),
            text(" and "),
            text("code", [{ type: "code" }]),
            text("."),
          ],
        },
      ],
    );
  });

  it("stacks nested marks", () => {
    const [block] = markdownToBlocks("***both***");
    const marks = block!.content![0]!.marks!.map((m) => m.type).sort();
    expect(marks).toEqual(["bold", "italic"]);
  });

  it("shifts headings down one level and clamps at h4", () => {
    const levels = markdownToBlocks(
      "# A\n\n## B\n\n### C\n\n#### D\n\n##### E",
    ).map((n) => n.attrs?.level);
    expect(levels).toEqual([2, 3, 4, 4, 4]);
  });

  it("wraps tight list items in paragraphs", () => {
    expect(markdownToBlocks("- one\n- two")).toEqual([
      {
        type: "bulletList",
        content: [
          {
            type: "listItem",
            content: [{ type: "paragraph", content: [text("one")] }],
          },
          {
            type: "listItem",
            content: [{ type: "paragraph", content: [text("two")] }],
          },
        ],
      },
    ]);
  });

  it("keeps an ordered list start only when it is not 1", () => {
    expect(markdownToBlocks("1. a\n2. b")[0]).toMatchObject({
      type: "orderedList",
    });
    expect(markdownToBlocks("1. a\n2. b")[0]!.attrs).toBeUndefined();
    expect(markdownToBlocks("3. a\n4. b")[0]!.attrs).toEqual({ start: 3 });
  });

  it("keeps the fence language and omits it when absent", () => {
    expect(markdownToBlocks("```rust\nfn main() {}\n```")).toEqual([
      {
        type: "codeBlock",
        attrs: { language: "rust" },
        content: [text("fn main() {}")],
      },
    ]);
    const [plain] = markdownToBlocks("```\nplain\n```");
    expect(plain!.attrs).toBeUndefined();
    expect(plain!.content).toEqual([text("plain")]);
  });

  it("turns a link into a link mark with only an href", () => {
    expect(markdownToBlocks("see [the site](https://example.com)")).toEqual([
      {
        type: "paragraph",
        content: [
          text("see "),
          text("the site", [
            { type: "link", attrs: { href: "https://example.com" } },
          ]),
        ],
      },
    ]);
  });

  it("makes a hard break a hardBreak node and joins soft breaks with a space", () => {
    expect(markdownToBlocks("a  \nb")[0]!.content).toEqual([
      text("a"),
      { type: "hardBreak" },
      text("b"),
    ]);
    expect(markdownToBlocks("line one\nline two")[0]!.content).toEqual([
      text("line one line two"),
    ]);
  });

  it("emits horizontal rules and blockquotes", () => {
    expect(markdownToBlocks("---")[0]!.type).toBe("horizontalRule");
    expect(markdownToBlocks("> quoted")).toEqual([
      {
        type: "blockquote",
        content: [{ type: "paragraph", content: [text("quoted")] }],
      },
    ]);
  });

  it("keeps multiple top-level blocks in order", () => {
    const types = markdownToBlocks("# Title\n\nfirst\n\nsecond").map(
      (n) => n.type,
    );
    expect(types).toEqual(["heading", "paragraph", "paragraph"]);
  });

  it("rejects empty and whitespace-only input", () => {
    for (const input of ["", "   \n  "]) {
      expect(() => markdownToBlocks(input)).toThrow(
        expect.objectContaining({ kind: "empty" }),
      );
    }
  });

  it.each([
    ["![alt](photo.png)", "images"],
    ["<div>hi</div>", "raw HTML"],
    ["a <b>bold</b> b", "raw HTML"],
    ["| a | b |\n|---|---|\n| 1 | 2 |", "tables"],
    ["- [ ] task", "task lists"],
  ])("rejects unsupported construct %j", (input, what) => {
    expect(() => markdownToBlocks(input)).toThrow(MarkdownError);
    expect(() => markdownToBlocks(input)).toThrow(
      `${what} are not supported in detail content`,
    );
  });
});

describe("inline code hints", () => {
  it("sets lang from a {:lang} suffix", () => {
    expect(markdownToBlocks("run `let x = 1{:ts}` now")[0]!.content).toEqual([
      text("run "),
      text("let x = 1", [{ type: "code", attrs: { lang: "ts" } }]),
      text(" now"),
    ]);
  });

  it("sets token from a {:.kind} suffix", () => {
    expect(markdownToBlocks("`Arc{:.type}`")[0]!.content).toEqual([
      text("Arc", [{ type: "code", attrs: { token: "type" } }]),
    ]);
  });

  it.each([
    ["obj{foo}", "no colon sigil"],
    ["{:rust}", "nothing before the suffix"],
    ["x{:.}", "empty token kind"],
    ["x{:rüst}", "non-ascii spec"],
  ])("leaves %j literal (%s)", (code) => {
    expect(splitCodeHint(code)).toEqual({ text: code });
  });

  it("treats any shape-valid bare suffix as a lang", () => {
    expect(splitCodeHint("obj{:foo}")).toEqual({
      text: "obj",
      attrs: { lang: "foo" },
    });
  });

  it("takes only the last suffix", () => {
    expect(splitCodeHint("a{:b}{:rust}")).toEqual({
      text: "a{:b}",
      attrs: { lang: "rust" },
    });
  });
});
