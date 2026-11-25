import { describe, expect, it } from "vitest";
import { uniqueIdOptions } from "./extensions.ts";
import {
  BLOCK_TYPES,
  OpError,
  applyAll,
  applyOp,
  blockId,
  deleteBlock,
  directText,
  docAt,
  docFromStored,
  emptyDoc,
  ensureBlockIds,
  generateBlockId,
  idLocator,
  insertBlock,
  insertSequence,
  moveBlock,
  nodeAtPath,
  outline,
  parseAnchor,
  parseBlockPath,
  parseLocator,
  replaceBlock,
  replaceSequence,
  type Anchor,
  type DocNode,
  type DocOp,
  type Locator,
} from "./ops.ts";

const para = (text: string): DocNode => ({
  type: "paragraph",
  content: [{ type: "text", text }],
});

const listItem = (text: string): DocNode => ({
  type: "listItem",
  content: [para(text)],
});

const id = idLocator;
const loc = (s: string): Locator => parseLocator(s);
const after = (target: Locator | string): Anchor => ({
  at: "after",
  id: typeof target === "string" ? id(target) : target,
});
const before = (target: string): Anchor => ({ at: "before", id: id(target) });
const END: Anchor = { at: "end" };
const START: Anchor = { at: "start" };

const ids = (doc: DocNode) =>
  (doc.content ?? []).map(blockId).filter((x) => x !== undefined);

function seed(): DocNode {
  const doc = emptyDoc();
  insertBlock(doc, END, para("one"), "a");
  insertBlock(doc, END, para("two"), "b");
  insertBlock(doc, END, para("three"), "c");
  return doc;
}

function counter() {
  let n = 0;
  return () => `g${++n}`;
}

describe("insert", () => {
  it("adds one block to an empty doc", () => {
    const doc = emptyDoc();
    const created = insertBlock(doc, END, para("hello"), "x1");
    expect(blockId(created)).toBe("x1");
    expect(created.type).toBe("paragraph");
    expect(ids(doc)).toEqual(["x1"]);
  });

  it("stamps the supplied id over any id the node carried", () => {
    const doc = emptyDoc();
    const incoming = { ...para("hi"), attrs: { id: "client-chosen" } };
    const created = insertBlock(doc, END, incoming, "server");
    expect(blockId(created)).toBe("server");
    expect(ids(doc)).toEqual(["server"]);
  });

  it("places after, before and at the start", () => {
    const afterDoc = seed();
    insertBlock(afterDoc, after("a"), para("mid"), "z");
    expect(ids(afterDoc)).toEqual(["a", "z", "b", "c"]);

    const beforeDoc = seed();
    insertBlock(beforeDoc, before("c"), para("mid"), "z");
    expect(ids(beforeDoc)).toEqual(["a", "b", "z", "c"]);

    const startDoc = seed();
    insertBlock(startDoc, START, para("first"), "z");
    expect(ids(startDoc)).toEqual(["z", "a", "b", "c"]);
  });

  it("rejects a missing anchor and a duplicate id", () => {
    const doc = seed();
    expect(() => insertBlock(doc, after("nope"), para("x"), "z")).toThrow(
      new OpError("not_found", 'block "nope" does not exist'),
    );
    expect(() => insertBlock(doc, END, para("x"), "b")).toThrow(
      new OpError("duplicate_id", 'block id "b" already exists'),
    );
  });
});

describe("replace", () => {
  it("swaps the node keeping id and position", () => {
    const doc = seed();
    const heading: DocNode = {
      type: "heading",
      attrs: { level: 2 },
      content: [{ type: "text", text: "H" }],
    };
    const replaced = replaceBlock(doc, id("b"), heading);
    expect(blockId(replaced)).toBe("b");
    expect(ids(doc)).toEqual(["a", "b", "c"]);
    expect(docAt(doc, id("b"))!.type).toBe("heading");
  });

  it("preserves the target id over the node's own id", () => {
    const doc = seed();
    replaceBlock(doc, id("b"), { ...para("x"), attrs: { id: "wrong" } });
    expect(ids(doc)).toEqual(["a", "b", "c"]);
    expect(docAt(doc, id("wrong"))).toBeUndefined();
  });

  it("reports a missing block", () => {
    expect(() => replaceBlock(seed(), id("nope"), para("x"))).toThrow(
      'block "nope" does not exist',
    );
  });
});

describe("delete", () => {
  it("removes the block and returns it", () => {
    const doc = seed();
    const removed = deleteBlock(doc, id("b"));
    expect(blockId(removed)).toBe("b");
    expect(ids(doc)).toEqual(["a", "c"]);
  });

  it("reports a missing block", () => {
    expect(() => deleteBlock(seed(), id("nope"))).toThrow(
      new OpError("not_found", 'block "nope" does not exist'),
    );
  });
});

describe("move", () => {
  it.each([
    ["a", after("b"), ["b", "a", "c"]],
    ["c", before("a"), ["c", "a", "b"]],
    ["a", END, ["b", "c", "a"]],
    ["c", START, ["c", "a", "b"]],
  ] as const)("moves %s to %j", (target, anchor, expected) => {
    const doc = seed();
    moveBlock(doc, id(target), anchor);
    expect(ids(doc)).toEqual(expected);
  });

  it("rejects an anchor on itself", () => {
    expect(() => moveBlock(seed(), id("b"), after("b"))).toThrow(
      new OpError(
        "self_anchor",
        "a block cannot anchor to itself or its own descendant",
      ),
    );
  });

  it("reports a missing block or anchor", () => {
    expect(() => moveBlock(seed(), id("nope"), END)).toThrow(
      'block "nope" does not exist',
    );
    expect(() => moveBlock(seed(), id("a"), after("nope"))).toThrow(
      'block "nope" does not exist',
    );
  });
});

describe("applyOp", () => {
  it("uses the generated id for an insert", () => {
    const doc = emptyDoc();
    const created = applyOp(
      doc,
      { op: "insert", anchor: END, node: para("hi") },
      () => "gen1",
    );
    expect(blockId(created)).toBe("gen1");
    expect(ids(doc)).toEqual(["gen1"]);
  });

  it("does not generate an id for replace, delete or move", () => {
    const unreachable = () => {
      throw new Error("must not generate an id");
    };
    const doc = seed();
    applyOp(
      doc,
      { op: "replace", id: id("b"), node: para("changed") },
      unreachable,
    );
    applyOp(doc, { op: "move", id: id("a"), anchor: END }, unreachable);
    applyOp(doc, { op: "delete", id: id("c") }, unreachable);
    expect(ids(doc)).toEqual(["b", "a"]);
  });
});

describe("applyAll", () => {
  it("applies ops in order", () => {
    const ops: DocOp[] = [
      { op: "insert", anchor: after("a"), node: para("x") },
      { op: "delete", id: id("c") },
      { op: "replace", id: id("b"), node: para("B!") },
    ];
    const result = applyAll(seed(), ops, counter());
    expect(ids(result)).toEqual(["a", "g1", "b"]);
  });

  it("leaves the input untouched when an op fails", () => {
    const doc = seed();
    const ops: DocOp[] = [
      { op: "delete", id: id("a") },
      { op: "delete", id: id("missing") },
    ];
    expect(() => applyAll(doc, ops)).toThrow('block "missing" does not exist');
    expect(ids(doc)).toEqual(["a", "b", "c"]);
  });

  it("treats an empty batch as a no-op", () => {
    expect(ids(applyAll(seed(), []))).toEqual(["a", "b", "c"]);
  });

  it("lets a later op reference a block inserted earlier in the batch", () => {
    const ops: DocOp[] = [
      { op: "insert", anchor: END, node: para("first") },
      { op: "replace", id: id("g1"), node: para("edited") },
    ];
    const result = applyAll(emptyDoc(), ops, counter());
    expect(ids(result)).toEqual(["g1"]);
    expect(directText(result.content![0]!)).toBe("edited");
  });

  it("assigns distinct ids to multiple inserts", () => {
    const ops: DocOp[] = [
      { op: "insert", anchor: END, node: para("one") },
      { op: "insert", anchor: END, node: para("two") },
    ];
    expect(ids(applyAll(emptyDoc(), ops, counter()))).toEqual(["g1", "g2"]);
  });
});

describe("anchor parsing", () => {
  it("parses keywords and relative forms", () => {
    expect(parseAnchor("start")).toEqual(START);
    expect(parseAnchor("end")).toEqual(END);
    expect(parseAnchor("after:b")).toEqual(after("b"));
    expect(parseAnchor("before:c")).toEqual(before("c"));
  });

  it("handles prepend, append and the into alias", () => {
    expect(parseAnchor("prepend:.1")).toEqual({
      at: "prepend_to",
      id: loc(".1"),
    });
    expect(parseAnchor("append:.1")).toEqual({
      at: "append_to",
      id: loc(".1"),
    });
    expect(parseAnchor("into:abc")).toEqual({ at: "append_to", id: id("abc") });
  });

  it("rejects garbage and empty locators", () => {
    for (const bad of [
      "sideways",
      "after",
      "after:",
      "before:",
      "sideways:.1",
    ]) {
      expect(() => parseAnchor(bad)).toThrow();
    }
  });
});

function nestedDoc(): DocNode {
  const block = (
    type: string,
    attrId: string,
    content: DocNode[],
  ): DocNode => ({ type, attrs: { id: attrId }, content });
  const p = (attrId: string, t: string) =>
    block("paragraph", attrId, [{ type: "text", text: t }]);
  return {
    type: "doc",
    content: [
      p("p", "top"),
      block("bulletList", "list", [
        block("listItem", "item1", [p("ip1", "one")]),
        block("listItem", "item2", [p("ip2", "two")]),
      ]),
      block("blockquote", "quote", [p("qp", "q")]),
    ],
  };
}

const childIds = (doc: DocNode, parent: string) =>
  (docAt(doc, id(parent))!.content ?? []).map(blockId);

describe("nested blocks", () => {
  it("deletes a nested block", () => {
    const doc = nestedDoc();
    expect(blockId(deleteBlock(doc, id("item1")))).toBe("item1");
    expect(childIds(doc, "list")).toEqual(["item2"]);
    expect(ids(doc)).toEqual(["p", "list", "quote"]);
  });

  it("replaces a nested block keeping its id", () => {
    const doc = nestedDoc();
    const heading: DocNode = {
      type: "heading",
      attrs: { level: 3 },
      content: [{ type: "text", text: "H" }],
    };
    expect(blockId(replaceBlock(doc, id("ip1"), heading))).toBe("ip1");
    expect(docAt(doc, id("ip1"))!.type).toBe("heading");
    expect(childIds(doc, "item1")).toEqual(["ip1"]);
  });

  it("inserts after a nested block into that parent", () => {
    const doc = nestedDoc();
    insertBlock(doc, after("item1"), listItem("new"), "item1b");
    expect(childIds(doc, "list")).toEqual(["item1", "item1b", "item2"]);
  });

  it("moves a block across parents", () => {
    const doc = nestedDoc();
    moveBlock(doc, id("p"), after("qp"));
    expect(ids(doc)).toEqual(["list", "quote"]);
    expect(childIds(doc, "quote")).toEqual(["qp", "p"]);
  });

  it("lifts a deeply nested block to the top level", () => {
    const doc = nestedDoc();
    moveBlock(doc, id("ip1"), END);
    expect(childIds(doc, "item1")).toEqual([]);
    expect(ids(doc)).toEqual(["p", "list", "quote", "ip1"]);
  });

  it("rejects a move into the block's own subtree", () => {
    expect(() => moveBlock(nestedDoc(), id("list"), after("item1"))).toThrow(
      expect.objectContaining({ kind: "self_anchor" }),
    );
  });

  it("finds a nested block by id", () => {
    expect(docAt(nestedDoc(), id("ip2"))!.type).toBe("paragraph");
  });
});

describe("block paths and locators", () => {
  const pathDoc = (): DocNode => ({
    type: "doc",
    content: [
      para("intro"),
      { type: "bulletList", content: [listItem("one"), listItem("two")] },
      { type: "blockquote", content: [para("quoted")] },
    ],
  });
  const paraTexts = (doc: DocNode) =>
    outline(doc)
      .filter(({ node }) => node.type === "paragraph")
      .map(({ node }) => directText(node));

  it("accepts dotted paths with or without the leading dot", () => {
    expect(parseBlockPath(".3")).toEqual([3]);
    expect(parseBlockPath("3")).toEqual([3]);
    expect(parseBlockPath(".3.0.1")).toEqual([3, 0, 1]);
  });

  it("accepts jq bracket forms", () => {
    expect(parseBlockPath("[3][0]")).toEqual([3, 0]);
    expect(parseBlockPath(".content[3].content[0]")).toEqual([3, 0]);
  });

  it("rejects non-numeric segments and empty paths", () => {
    for (const bad of ["a1b2c3d4", ".3.x", ".", ""]) {
      expect(() => parseBlockPath(bad)).toThrow();
    }
  });

  it("disambiguates a path from an id", () => {
    expect(loc(".3.0")).toEqual({ kind: "path", path: [3, 0] });
    expect(loc("a1b2c3d4")).toEqual(id("a1b2c3d4"));
    expect(() => parseLocator("")).toThrow();
  });

  it("accepts the hash-prefixed id that list prints", () => {
    expect(loc("#a1b2c3d4")).toEqual(id("a1b2c3d4"));
    expect(() => parseLocator("#")).toThrow();
  });

  it("resolves both locator forms and the hash form", () => {
    const doc = emptyDoc();
    insertBlock(doc, END, para("hello"), "keepid12");
    expect(directText(docAt(doc, loc(".0"))!)).toBe("hello");
    expect(directText(docAt(doc, loc("keepid12"))!)).toBe("hello");
    expect(directText(docAt(doc, loc("#keepid12"))!)).toBe("hello");
    expect(docAt(doc, loc("nope0000"))).toBeUndefined();
    expect(docAt(doc, loc(".9"))).toBeUndefined();
  });

  it("resolves top-level and nested paths and fails cleanly out of range", () => {
    const doc = pathDoc();
    expect(nodeAtPath(doc, [1])!.type).toBe("bulletList");
    expect(directText(nodeAtPath(doc, [1, 1, 0])!)).toBe("two");
    expect(nodeAtPath(doc, [9])).toBeUndefined();
    expect(nodeAtPath(doc, [1, 0, 5])).toBeUndefined();
  });

  it("outlines every block in pre-order without inline nodes", () => {
    const flat = outline(pathDoc()).map(
      ({ path, node }) => `.${path.join(".")} ${node.type}`,
    );
    expect(flat).toEqual([
      ".0 paragraph",
      ".1 bulletList",
      ".1.0 listItem",
      ".1.0.0 paragraph",
      ".1.1 listItem",
      ".1.1.0 paragraph",
      ".2 blockquote",
      ".2.0 paragraph",
    ]);
  });

  it("gives container blocks an empty direct text", () => {
    const doc = pathDoc();
    expect(directText(nodeAtPath(doc, [1])!)).toBe("");
    expect(directText(nodeAtPath(doc, [1, 0, 0])!)).toBe("one");
  });

  it("replaces by path on id-less content and keeps an existing id", () => {
    const doc = pathDoc();
    replaceBlock(doc, loc(".0"), para("changed"));
    expect(directText(nodeAtPath(doc, [0])!)).toBe("changed");

    const stamped = emptyDoc();
    insertBlock(stamped, END, para("first"), "keep");
    replaceBlock(stamped, loc(".0"), para("edited"));
    expect(blockId(nodeAtPath(stamped, [0])!)).toBe("keep");
  });

  it("deletes by nested path", () => {
    const doc = pathDoc();
    deleteBlock(doc, loc(".1.0"));
    expect(paraTexts(doc)).toEqual(["intro", "two", "quoted"]);
  });

  it("appends and prepends to a container", () => {
    const appended = pathDoc();
    insertBlock(
      appended,
      { at: "append_to", id: loc(".1") },
      listItem("three"),
      "x",
    );
    expect(directText(nodeAtPath(appended, [1, 2, 0])!)).toBe("three");

    const prepended = pathDoc();
    insertBlock(
      prepended,
      { at: "prepend_to", id: loc(".1") },
      listItem("zero"),
      "x",
    );
    expect(directText(nodeAtPath(prepended, [1, 0, 0])!)).toBe("zero");
  });

  it("resolves a move by path against the pre-move tree", () => {
    const doc = emptyDoc();
    for (const t of ["a", "b", "c"]) insertBlock(doc, END, para(t), t);
    moveBlock(doc, loc(".0"), { at: "after", id: loc(".2") });
    expect(paraTexts(doc)).toEqual(["b", "c", "a"]);
  });

  it("rejects appending a block into its own subtree", () => {
    expect(() =>
      moveBlock(pathDoc(), loc(".1"), { at: "append_to", id: loc(".1") }),
    ).toThrow(expect.objectContaining({ kind: "self_anchor" }));
  });

  it("does not mutate a path locator while moving", () => {
    const doc = pathDoc();
    const target = loc(".1");
    const anchor: Anchor = { at: "after", id: loc(".2") };
    moveBlock(doc, target, anchor);
    expect(anchor).toEqual({ at: "after", id: loc(".2") });
    expect(target).toEqual(loc(".1"));
  });
});

describe("sequences", () => {
  const seeded = (texts: string[]) => {
    const doc = emptyDoc();
    for (const t of texts) insertBlock(doc, END, para(t), t);
    return doc;
  };
  const texts = (doc: DocNode) =>
    outline(doc)
      .filter(({ node }) => node.type === "paragraph")
      .map(({ node }) => directText(node));
  const abc = () => [para("a"), para("b"), para("c")];

  it.each([
    ["end", END, ["x", "a", "b", "c"]],
    ["start", START, ["a", "b", "c", "x"]],
  ] as const)(
    "inserts a sequence at %s in order",
    (_name, anchor, expected) => {
      const result = applyAll(
        seeded(["x"]),
        insertSequence(anchor, abc()),
        counter(),
      );
      expect(texts(result)).toEqual(expected);
    },
  );

  it("inserts a sequence after and before a block in order", () => {
    expect(
      texts(
        applyAll(
          seeded(["x", "y"]),
          insertSequence(after("x"), abc()),
          counter(),
        ),
      ),
    ).toEqual(["x", "a", "b", "c", "y"]);
    expect(
      texts(
        applyAll(
          seeded(["x", "y"]),
          insertSequence(before("y"), abc()),
          counter(),
        ),
      ),
    ).toEqual(["x", "a", "b", "c", "y"]);
  });

  it("inserts a sequence into a container in order", () => {
    const items = () => [listItem("a"), listItem("b"), listItem("c")];
    const appendDoc = emptyDoc();
    insertBlock(appendDoc, END, { type: "bulletList", content: [] }, "list");
    expect(
      texts(
        applyAll(
          appendDoc,
          insertSequence({ at: "append_to", id: id("list") }, items()),
          counter(),
        ),
      ),
    ).toEqual(["a", "b", "c"]);

    const prependDoc = emptyDoc();
    insertBlock(
      prependDoc,
      END,
      { type: "bulletList", content: [listItem("z")] },
      "list",
    );
    expect(
      texts(
        applyAll(
          prependDoc,
          insertSequence({ at: "prepend_to", id: id("list") }, items()),
          counter(),
        ),
      ),
    ).toEqual(["a", "b", "c", "z"]);
  });

  it("makes a sequence of one a single insert", () => {
    const ops = insertSequence(END, [para("only")]);
    expect(ops).toHaveLength(1);
    expect(ops[0]!.op).toBe("insert");
  });

  it("replaces one block with a sequence, keeping the first in the target slot", () => {
    const single = applyAll(
      seeded(["x", "y"]),
      replaceSequence(id("x"), [para("a")]),
      counter(),
    );
    expect(texts(single)).toEqual(["a", "y"]);
    expect(directText(docAt(single, id("x"))!)).toBe("a");

    const multi = applyAll(
      seeded(["x", "y"]),
      replaceSequence(id("x"), abc()),
      counter(),
    );
    expect(texts(multi)).toEqual(["a", "b", "c", "y"]);
    expect(directText(docAt(multi, id("x"))!)).toBe("a");

    const byPath = applyAll(
      seeded(["x", "y"]),
      replaceSequence(loc(".0"), [para("a"), para("b")]),
      counter(),
    );
    expect(texts(byPath)).toEqual(["a", "b", "y"]);
  });

  it("yields no ops for an empty replacement", () => {
    expect(replaceSequence(id("x"), [])).toEqual([]);
  });
});

describe("block ids", () => {
  it("stamps every block-group node lacking one but not text or the root", () => {
    const doc: DocNode = {
      type: "doc",
      content: [para("p"), { type: "bulletList", content: [listItem("i")] }],
    };
    ensureBlockIds(doc, counter());
    const stamped = outline(doc).filter(({ node }) => blockId(node));
    expect(stamped).toHaveLength(4);
    expect(blockId(doc)).toBeUndefined();
    expect(blockId(doc.content![0]!.content![0]!)).toBeUndefined();
  });

  it("preserves existing ids and skips collisions", () => {
    const doc: DocNode = {
      type: "doc",
      content: [{ ...para("a"), attrs: { id: "g1" } }, para("b")],
    };
    ensureBlockIds(doc, counter());
    expect(blockId(doc.content![0]!)).toBe("g1");
    expect(blockId(doc.content![1]!)).toBe("g2");
  });

  it("backfills ids across a whole id-less document on apply", () => {
    const doc: DocNode = { type: "doc", content: [para("seed")] };
    const result = applyAll(
      doc,
      [{ op: "insert", anchor: END, node: para("added") }],
      counter(),
    );
    expect(result.content!.every((b) => blockId(b))).toBe(true);
  });

  it("stamps nested blocks of an inserted container", () => {
    const list: DocNode = { type: "bulletList", content: [listItem("x")] };
    const result = applyAll(
      emptyDoc(),
      [{ op: "insert", anchor: END, node: list }],
      counter(),
    );
    const bullet = result.content![0]!;
    expect(blockId(bullet)).toBeDefined();
    expect(blockId(bullet.content![0]!)).toBeDefined();
    expect(blockId(bullet.content![0]!.content![0]!)).toBeDefined();
  });

  it("generates eight lowercase alphanumerics without collisions", () => {
    expect(generateBlockId()).toMatch(/^[a-z0-9]{8}$/);
    const set = new Set(Array.from({ length: 1000 }, () => generateBlockId()));
    expect(set.size).toBe(1000);
  });

  it("stamps exactly the types the editor's unique-id extension does", () => {
    expect([...BLOCK_TYPES].sort()).toEqual(
      [...(uniqueIdOptions?.types as string[])].sort(),
    );
  });
});

describe("stored documents", () => {
  it("degrades anything that is not a doc to an empty doc", () => {
    expect(docFromStored(null)).toEqual(emptyDoc());
    expect(docFromStored("nonsense")).toEqual(emptyDoc());
    expect(docFromStored({ type: "paragraph", content: [] })).toEqual(
      emptyDoc(),
    );
  });

  it("keeps a valid doc and does not alias the input", () => {
    const stored = seed();
    const parsed = docFromStored(stored);
    expect(parsed).toEqual(stored);
    parsed.content!.pop();
    expect(stored.content).toHaveLength(3);
  });
});
