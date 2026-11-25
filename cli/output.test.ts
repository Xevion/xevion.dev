import { describe, expect, it } from "vitest";
import { blockRows } from "./output.ts";
import type { DocNode } from "#lib/tiptap/ops.ts";

const doc = (content: DocNode[]): DocNode => ({ type: "doc", content });
const text = (t: string): DocNode => ({ type: "text", text: t });

describe("blockRows", () => {
  it("reports id, path, type and tree indentation", () => {
    const rows = blockRows(
      doc([
        {
          type: "paragraph",
          attrs: { id: "aaaa1111" },
          content: [text("hello")],
        },
        {
          type: "bulletList",
          content: [
            {
              type: "listItem",
              content: [{ type: "paragraph", content: [text("nested")] }],
            },
          ],
        },
      ]),
    );
    expect(rows).toEqual([
      { id: "aaaa1111", path: ".0", typeLabel: "paragraph", preview: "hello" },
      { id: undefined, path: ".1", typeLabel: "bulletList", preview: "" },
      { id: undefined, path: ".1.0", typeLabel: "  listItem", preview: "" },
      {
        id: undefined,
        path: ".1.0.0",
        typeLabel: "    paragraph",
        preview: "nested",
      },
    ]);
  });

  it("truncates a long first line with an ellipsis", () => {
    const rows = blockRows(
      doc([{ type: "paragraph", content: [text("x".repeat(80))] }]),
    );
    expect(Array.from(rows[0]!.preview)).toHaveLength(61);
    expect(rows[0]!.preview.endsWith("…")).toBe(true);
  });
});
