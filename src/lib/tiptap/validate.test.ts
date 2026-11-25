import { describe, expect, it } from "vitest";
import {
  MAX_CODE_TEXT_CHARS,
  blockIdTypes,
  ensureBlockIds,
  generateBlockId,
  markTypeNames,
  nodeTypeNames,
  prepareDoc,
  toStored,
  validateDoc,
  type PmNode,
} from "./validate.ts";

const text = (value: string, marks?: PmNode["marks"]): PmNode =>
  marks ? { type: "text", text: value, marks } : { type: "text", text: value };
const para = (...content: PmNode[]): PmNode => ({ type: "paragraph", content });
const doc = (...content: PmNode[]): PmNode => ({ type: "doc", content });

function failure(value: unknown) {
  const result = validateDoc(value);
  if (result.ok) throw new Error("expected validation to fail");
  return result;
}

function counter() {
  let n = 0;
  return () => `g${++n}`;
}

function idsOf(node: PmNode): string[] {
  const own = typeof node.attrs?.id === "string" ? [node.attrs.id] : [];
  return [...own, ...(node.content ?? []).flatMap(idsOf)];
}

const sample = doc(
  {
    type: "heading",
    attrs: { level: 2 },
    content: [text("Title")],
  },
  para(
    text("plain "),
    text("bold", [{ type: "bold" }]),
    text(" and "),
    text("link", [{ type: "link", attrs: { href: "https://example.com" } }]),
    { type: "hardBreak" },
  ),
  {
    type: "bulletList",
    content: [{ type: "listItem", content: [para(text("item"))] }],
  },
  { type: "blockquote", content: [para(text("quote"))] },
  {
    type: "codeBlock",
    attrs: { language: "rust" },
    content: [text("fn main() {}")],
  },
  { type: "horizontalRule" },
);

describe("validateDoc schema", () => {
  it("accepts a representative document unchanged", () => {
    const result = validateDoc(sample);
    expect(result).toEqual({ ok: true, doc: sample });
  });

  it("derives the node and mark allow-lists from the editor schema", () => {
    expect([...nodeTypeNames].sort()).toEqual(
      [
        "blockquote",
        "bulletList",
        "callout",
        "codeBlock",
        "details",
        "doc",
        "figure",
        "hardBreak",
        "heading",
        "horizontalRule",
        "listItem",
        "orderedList",
        "paragraph",
        "sidenote",
        "text",
      ].sort(),
    );
    expect([...markTypeNames].sort()).toEqual(
      ["bold", "code", "gloss", "italic", "link", "strike", "underline"].sort(),
    );
    expect([...blockIdTypes].sort()).toEqual(
      [
        "blockquote",
        "bulletList",
        "callout",
        "codeBlock",
        "details",
        "figure",
        "heading",
        "horizontalRule",
        "listItem",
        "orderedList",
        "paragraph",
        "sidenote",
      ].sort(),
    );
  });

  it("accepts an empty doc, with or without content", () => {
    expect(validateDoc({ type: "doc" }).ok).toBe(true);
    expect(validateDoc({ type: "doc", content: [] }).ok).toBe(true);
  });

  it("accepts the custom widget nodes", () => {
    const result = validateDoc(
      doc(
        { type: "figure", attrs: { src: "/a.png", alt: "a", kind: "image" } },
        { type: "callout", attrs: { variant: "tip" }, content: [para()] },
        { type: "sidenote", content: [para(text("aside"))] },
        {
          type: "details",
          attrs: { summary: "More", open: false },
          content: [para(text("body"))],
        },
        para(text("hint", [{ type: "gloss", attrs: { note: "n" } }])),
      ),
    );
    expect(result.ok).toBe(true);
  });

  it("rejects non-doc and non-object roots", () => {
    expect(failure({ type: "paragraph", content: [] })).toMatchObject({
      kind: "not-a-doc",
      path: [],
    });
    expect(failure("nonsense").kind).toBe("malformed");
    expect(failure(null).kind).toBe("malformed");
    expect(failure({ type: "doc", content: "x" }).kind).toBe("malformed");
  });

  it("rejects an unknown node and reports its path", () => {
    const result = failure(doc(para(text("ok")), { type: "script" }));
    expect(result).toMatchObject({ kind: "unknown-node", path: [1] });
    expect(result.message).toContain('unknown node type "script"');
    expect(result.message).toContain(".1");
  });

  it("does not mistake Object.prototype keys for node or mark types", () => {
    expect(failure(doc({ type: "constructor" })).kind).toBe("unknown-node");
    expect(failure(doc(para(text("x", [{ type: "toString" }])))).kind).toBe(
      "unknown-mark",
    );
  });

  it("rejects an unknown mark", () => {
    expect(failure(doc(para(text("x", [{ type: "blink" }]))))).toMatchObject({
      kind: "unknown-mark",
      path: [0, 0],
    });
  });

  it("rejects marks on a non-text node", () => {
    expect(
      failure(doc({ type: "paragraph", marks: [{ type: "bold" }] })),
    ).toMatchObject({ kind: "marks-on-non-text", path: [0] });
  });

  it("rejects marks inside a code block", () => {
    expect(
      failure(
        doc({
          type: "codeBlock",
          content: [text("x", [{ type: "bold" }])],
        }),
      ),
    ).toMatchObject({ kind: "marks-in-code-block", path: [0, 0] });
  });

  it("rejects empty text, text with content, and text on a non-text node", () => {
    expect(failure(doc(para(text("")))).kind).toBe("empty-text");
    expect(
      failure(doc(para({ ...text("x"), content: [text("y")] }))).kind,
    ).toBe("text-with-content");
    expect(
      failure(doc({ type: "paragraph", text: "oops", content: [] })),
    ).toMatchObject({ kind: "non-text-with-text" });
  });

  it("rejects a child its parent does not permit", () => {
    expect(
      failure(doc({ type: "bulletList", content: [para(text("x"))] })),
    ).toMatchObject({
      kind: "disallowed-child",
      path: [0, 0],
      message: expect.stringContaining(
        'node "paragraph" is not allowed inside "bulletList"',
      ),
    });
  });

  it("rejects an inline node at block level", () => {
    expect(failure(doc(text("x")))).toMatchObject({
      kind: "disallowed-child",
      path: [0],
    });
  });

  it("rejects children on a leaf", () => {
    expect(
      failure(doc({ type: "horizontalRule", content: [para(text("x"))] })),
    ).toMatchObject({ kind: "disallowed-child", path: [0, 0] });
  });

  it("rejects a container missing its required content", () => {
    expect(failure(doc({ type: "blockquote" }))).toMatchObject({
      kind: "incomplete-content",
      path: [0],
    });
  });

  it("rejects duplicated marks", () => {
    expect(
      failure(doc(para(text("x", [{ type: "bold" }, { type: "bold" }])))).kind,
    ).toBe("invalid-marks");
  });

  it("canonicalizes mark order and drops unknown fields and empty containers", () => {
    const result = validateDoc({
      type: "doc",
      attrs: {},
      stray: true,
      content: [
        {
          type: "paragraph",
          content: [
            {
              type: "text",
              text: "x",
              marks: [
                { type: "bold" },
                { type: "link", attrs: { href: "https://a.test" } },
              ],
            },
          ],
        },
      ],
    });
    assertOk(result);
    const marks = result.doc.content?.[0]?.content?.[0]?.marks?.map(
      (m) => m.type,
    );
    expect(marks).toEqual(["link", "bold"]);
    expect(result.doc).not.toHaveProperty("stray");
    expect(result.doc).not.toHaveProperty("attrs");
  });

  it("rejects a document nested past the depth limit", () => {
    let node: PmNode = para(text("x"));
    for (let i = 0; i < 200; i++)
      node = { type: "blockquote", content: [node] };
    expect(failure(doc(node)).kind).toBe("malformed");
  });
});

function assertOk(
  result: ReturnType<typeof validateDoc>,
): asserts result is Extract<ReturnType<typeof validateDoc>, { ok: true }> {
  if (!result.ok) throw new Error(result.message);
}

describe("validateDoc link marks", () => {
  const link = (href: unknown) =>
    doc(para(text("x", [{ type: "link", attrs: { href } }])));

  it("rejects a disallowed scheme", () => {
    const result = failure(link("javascript:alert(1)"));
    expect(result.kind).toBe("bad-link-scheme");
    expect(result.message).toContain('"javascript:alert(1)"');
  });

  it("accepts http, https and mailto case-insensitively", () => {
    for (const href of [
      "http://a.test",
      "https://a.test/x?y=1",
      "HTTPS://A.TEST",
      "MailTo:me@a.test",
    ])
      expect(validateDoc(link(href)).ok, href).toBe(true);
  });

  it("rejects a missing, non-string, relative or padded href", () => {
    for (const href of [
      undefined,
      42,
      "/relative",
      " https://a.test",
      "data:x",
    ])
      expect(failure(link(href)).kind, String(href)).toBe("bad-link-scheme");
  });
});

describe("validateDoc code marks", () => {
  const code = (value: string, attrs?: Record<string, unknown>) =>
    doc(para(text(value, [{ type: "code", ...(attrs ? { attrs } : {}) }])));

  it("accepts a known token and a cleared token", () => {
    expect(validateDoc(code("Arc", { token: "type" })).ok).toBe(true);
    expect(validateDoc(code("x", { token: null, lang: "rust" })).ok).toBe(true);
  });

  it("rejects an unknown or non-string token", () => {
    expect(failure(code("x", { token: "bogus" }))).toMatchObject({
      kind: "bad-code-token",
      message: expect.stringContaining('"bogus"'),
    });
    expect(failure(code("x", { token: 42 })).message).toContain('"42"');
  });

  it("accepts any lang", () => {
    expect(validateDoc(code("x", { lang: "totally-made-up" })).ok).toBe(true);
  });

  it("caps a code-marked run, counting characters", () => {
    expect(validateDoc(code("x".repeat(MAX_CODE_TEXT_CHARS))).ok).toBe(true);
    expect(failure(code("x".repeat(MAX_CODE_TEXT_CHARS + 1)))).toMatchObject({
      kind: "code-too-long",
      message: expect.stringContaining(`${MAX_CODE_TEXT_CHARS + 1} characters`),
    });
    // Astral characters are one character each, not two UTF-16 units.
    expect(validateDoc(code("\u{1F600}".repeat(MAX_CODE_TEXT_CHARS))).ok).toBe(
      true,
    );
  });

  it("does not cap long text without a code mark", () => {
    expect(
      validateDoc(doc(para(text("x".repeat(MAX_CODE_TEXT_CHARS + 1))))).ok,
    ).toBe(true);
  });

  it("allows a code mark alongside a link", () => {
    expect(
      validateDoc(
        doc(
          para(
            text("x", [
              { type: "link", attrs: { href: "https://a.test" } },
              { type: "code" },
            ]),
          ),
        ),
      ).ok,
    ).toBe(true);
  });
});

describe("ensureBlockIds", () => {
  const fixture = doc(para(text("p")), {
    type: "bulletList",
    content: [{ type: "listItem", content: [para(text("i"))] }],
  });

  it("stamps every block node and never the root or inline nodes", () => {
    const stamped = ensureBlockIds(fixture, counter());
    expect(idsOf(stamped)).toHaveLength(4);
    expect(stamped.attrs).toBeUndefined();
    expect(stamped.content?.[0]?.content?.[0]?.attrs).toBeUndefined();
    expect(fixture.content?.[0]?.attrs).toBeUndefined();
  });

  it("preserves existing ids", () => {
    const stamped = ensureBlockIds(
      doc(
        { type: "paragraph", attrs: { id: "keep" }, content: [text("a")] },
        para(text("b")),
      ),
      counter(),
    );
    expect(stamped.content?.map((n) => n.attrs?.id)).toEqual(["keep", "g1"]);
  });

  it("skips a generated id that collides with an existing one", () => {
    const stamped = ensureBlockIds(
      doc(
        { type: "paragraph", attrs: { id: "g1" }, content: [text("a")] },
        para(text("b")),
      ),
      counter(),
    );
    expect(stamped.content?.[1]?.attrs?.id).toBe("g2");
  });

  it("keeps other attrs when stamping", () => {
    const stamped = ensureBlockIds(
      doc({ type: "heading", attrs: { level: 3 }, content: [text("h")] }),
      counter(),
    );
    expect(stamped.content?.[0]?.attrs).toEqual({ level: 3, id: "g1" });
  });

  it("mints eight lowercase alphanumerics without collisions", () => {
    const ids = new Set(Array.from({ length: 1000 }, generateBlockId));
    expect(ids.size).toBe(1000);
    for (const id of ids) expect(id).toMatch(/^[a-z0-9]{8}$/);
  });
});

describe("prepareDoc", () => {
  it("stores an empty document as null", () => {
    expect(prepareDoc({ type: "doc" })).toEqual({ ok: true, doc: null });
    expect(prepareDoc({ type: "doc", content: [] })).toEqual({
      ok: true,
      doc: null,
    });
    expect(toStored({ type: "doc" })).toBeNull();
  });

  it("validates, then stamps ids", () => {
    const result = prepareDoc(doc(para(text("a"))), counter());
    expect(result).toEqual({
      ok: true,
      doc: doc({
        type: "paragraph",
        attrs: { id: "g1" },
        content: [text("a")],
      }),
    });
  });

  it("surfaces validation failures", () => {
    expect(
      prepareDoc({ type: "doc", content: [{ type: "script" }] }),
    ).toMatchObject({ ok: false, kind: "unknown-node" });
  });
});
