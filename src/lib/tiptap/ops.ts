import { customAlphabet } from "nanoid";

export interface Mark {
  type: string;
  attrs?: Record<string, unknown>;
}

/** One ProseMirror node; the document root is a node of type "doc". */
export interface DocNode {
  type: string;
  attrs?: Record<string, unknown>;
  content?: DocNode[];
  marks?: Mark[];
  text?: string;
}

export const ID_ATTR = "id";

/** Block-group node types, exactly the set the editor's unique-id extension stamps. */
export const BLOCK_TYPES: ReadonlySet<string> = new Set([
  "paragraph",
  "heading",
  "blockquote",
  "codeBlock",
  "horizontalRule",
  "bulletList",
  "orderedList",
  "listItem",
  "figure",
  "sidenote",
  "callout",
  "details",
]);

const INLINE_TYPES: ReadonlySet<string> = new Set(["text", "hardBreak"]);

export const generateBlockId = customAlphabet(
  "abcdefghijklmnopqrstuvwxyz0123456789",
  8,
);

export type OpErrorKind = "not_found" | "duplicate_id" | "self_anchor";

export class OpError extends Error {
  constructor(
    readonly kind: OpErrorKind,
    message: string,
  ) {
    super(message);
    this.name = "OpError";
  }

  static notFound(locator: string): OpError {
    return new OpError("not_found", `block "${locator}" does not exist`);
  }

  static duplicateId(id: string): OpError {
    return new OpError("duplicate_id", `block id "${id}" already exists`);
  }

  static selfAnchor(): OpError {
    return new OpError(
      "self_anchor",
      "a block cannot anchor to itself or its own descendant",
    );
  }
}

/** Child-index path from the document root, e.g. [3, 0] is content[3].content[0]. */
export type BlockPath = number[];

export type Locator =
  | { kind: "id"; id: string }
  | { kind: "path"; path: BlockPath };

/** Dotted form with a leading dot, e.g. ".3.0". */
export function formatBlockPath(path: BlockPath): string {
  return path.map((i) => `.${i}`).join("");
}

/**
 * Accepts ".3.0", "3.0", "[3][0]" and ".content[3].content[0]". A non-numeric
 * segment (such as a block id) is rejected, which is how a path is told apart
 * from an id.
 */
export function parseBlockPath(s: string): BlockPath {
  const cleaned = s.replaceAll("content", " ");
  const segments = cleaned.split(/[.[\]\s]/).filter((seg) => seg.length > 0);
  const indices = segments.map((seg) => {
    if (!/^\d+$/.test(seg)) {
      throw new Error(`invalid path segment "${seg}" in "${s}"`);
    }
    return Number.parseInt(seg, 10);
  });
  if (indices.length === 0) {
    throw new Error(`empty path "${s}"; use indices like .3 or .3.0`);
  }
  return indices;
}

/**
 * A leading "." or "[" is a positional path; anything else is a block id. A
 * leading "#" on an id is stripped so the form `list` prints pastes back in.
 */
export function parseLocator(s: string): Locator {
  const empty = "empty locator; use a block id or a path like .3";
  if (s.length === 0) throw new Error(empty);
  if (s.startsWith(".") || s.startsWith("[")) {
    return { kind: "path", path: parseBlockPath(s) };
  }
  const id = s.startsWith("#") ? s.slice(1) : s;
  if (id.length === 0) throw new Error(empty);
  return { kind: "id", id };
}

export function locatorToString(locator: Locator): string {
  return locator.kind === "id" ? locator.id : formatBlockPath(locator.path);
}

/** Shorthand for an id locator. */
export function idLocator(id: string): Locator {
  return { kind: "id", id };
}

/**
 * Where an insert or move lands. start/end address the top-level block list;
 * before/after land beside a block within its own parent; prepend/append land
 * inside a container as its first/last child.
 */
export type Anchor =
  | { at: "start" }
  | { at: "end" }
  | { at: "after"; id: Locator }
  | { at: "before"; id: Locator }
  | { at: "prepend_to"; id: Locator }
  | { at: "append_to"; id: Locator };

function invalidAnchor(s: string): Error {
  return new Error(
    `invalid position "${s}"; use start, end, before:<loc>, after:<loc>, ` +
      "prepend:<loc>, or append:<loc>",
  );
}

/** `start`, `end`, or `<kind>:<locator>`; `into` is an alias for `append`. */
export function parseAnchor(s: string): Anchor {
  if (s === "start") return { at: "start" };
  if (s === "end") return { at: "end" };
  const colon = s.indexOf(":");
  if (colon < 0) throw invalidAnchor(s);
  const kind = s.slice(0, colon);
  const id = parseLocator(s.slice(colon + 1));
  switch (kind) {
    case "after":
      return { at: "after", id };
    case "before":
      return { at: "before", id };
    case "prepend":
      return { at: "prepend_to", id };
    case "append":
    case "into":
      return { at: "append_to", id };
    default:
      throw invalidAnchor(s);
  }
}

/**
 * Whether inserting several blocks one at a time at this anchor lands them in
 * reverse order: start, prepend and after each push the previous insert away.
 */
function fillsInReverse(anchor: Anchor): boolean {
  return (
    anchor.at === "start" || anchor.at === "prepend_to" || anchor.at === "after"
  );
}

export type DocOp =
  | { op: "insert"; anchor: Anchor; node: DocNode }
  | { op: "replace"; id: Locator; node: DocNode }
  | { op: "delete"; id: Locator }
  | { op: "move"; id: Locator; anchor: Anchor };

/** Ops that land `nodes` in document order at `anchor`. */
export function insertSequence(anchor: Anchor, nodes: DocNode[]): DocOp[] {
  const ordered = fillsInReverse(anchor) ? [...nodes].reverse() : nodes;
  return ordered.map((node) => ({ op: "insert", anchor, node }));
}

/** Replace `target` with the first node and insert the rest right after it. */
export function replaceSequence(target: Locator, nodes: DocNode[]): DocOp[] {
  const [first, ...rest] = nodes;
  if (first === undefined) return [];
  const ops: DocOp[] = [{ op: "replace", id: target, node: first }];
  if (rest.length > 0) {
    ops.push(...insertSequence({ at: "after", id: target }, rest));
  }
  return ops;
}

export function emptyDoc(): DocNode {
  return { type: "doc", content: [] };
}

/** Anything that is not a doc node (missing, garbage) degrades to an empty doc. */
export function docFromStored(value: unknown): DocNode {
  if (
    typeof value === "object" &&
    value !== null &&
    (value as { type?: unknown }).type === "doc"
  ) {
    const doc = structuredClone(value) as DocNode;
    doc.content ??= [];
    return doc;
  }
  return emptyDoc();
}

export function blockId(node: DocNode): string | undefined {
  const id = node.attrs?.[ID_ATTR];
  return typeof id === "string" ? id : undefined;
}

function setBlockId(node: DocNode, id: string): void {
  node.attrs = { ...node.attrs, [ID_ATTR]: id };
}

/** A node's own text from its direct text children only. */
export function directText(node: DocNode): string {
  return (node.content ?? [])
    .filter((c) => c.type === "text")
    .map((c) => c.text ?? "")
    .join("");
}

function isInline(node: DocNode): boolean {
  return INLINE_TYPES.has(node.type);
}

function isBlock(node: DocNode): boolean {
  return BLOCK_TYPES.has(node.type);
}

function children(node: DocNode): DocNode[] {
  return (node.content ??= []);
}

/** Path of child indices from `node` to the descendant carrying `id`. */
function findPath(node: DocNode, id: string): BlockPath | undefined {
  const kids = node.content ?? [];
  for (const [i, child] of kids.entries()) {
    if (blockId(child) === id) return [i];
    const sub = findPath(child, id);
    if (sub) return [i, ...sub];
  }
  return undefined;
}

export function nodeAtPath(doc: DocNode, path: BlockPath): DocNode | undefined {
  let node: DocNode | undefined = doc;
  for (const i of path) {
    node = node.content?.[i];
    if (node === undefined) return undefined;
  }
  return node;
}

/** The path of the block a locator names, or undefined if it names nothing. */
function resolveLocator(doc: DocNode, locator: Locator): BlockPath | undefined {
  if (locator.kind === "id") return findPath(doc, locator.id);
  return nodeAtPath(doc, locator.path) ? [...locator.path] : undefined;
}

/** The block a locator names, by id or positional path. */
export function docAt(doc: DocNode, locator: Locator): DocNode | undefined {
  const path = resolveLocator(doc, locator);
  return path ? nodeAtPath(doc, path) : undefined;
}

/** Pre-order walk of every non-inline node with its positional path. */
export function outline(doc: DocNode): { path: BlockPath; node: DocNode }[] {
  const out: { path: BlockPath; node: DocNode }[] = [];
  const walk = (parent: BlockPath, nodes: DocNode[]) => {
    for (const [i, node] of nodes.entries()) {
      if (isInline(node)) continue;
      const path = [...parent, i];
      out.push({ path, node });
      walk(path, node.content ?? []);
    }
  };
  walk([], doc.content ?? []);
  return out;
}

function contentAt(doc: DocNode, parentPath: BlockPath): DocNode[] {
  let node = doc;
  for (const i of parentPath) node = node.content![i]!;
  return children(node);
}

function startsWith(path: BlockPath, prefix: BlockPath): boolean {
  return (
    path.length >= prefix.length && prefix.every((seg, i) => path[i] === seg)
  );
}

function splitLast(path: BlockPath): [parent: BlockPath, index: number] {
  return [path.slice(0, -1), path[path.length - 1]!];
}

/** The parent whose content receives a node, and the index within it. */
function resolveAnchor(
  doc: DocNode,
  anchor: Anchor,
): [parent: BlockPath, index: number] {
  const locate = (locator: Locator): BlockPath => {
    const path = resolveLocator(doc, locator);
    if (!path) throw OpError.notFound(locatorToString(locator));
    return path;
  };
  switch (anchor.at) {
    case "start":
      return [[], 0];
    case "end":
      return [[], (doc.content ?? []).length];
    case "before": {
      const [parent, index] = splitLast(locate(anchor.id));
      return [parent, index];
    }
    case "after": {
      const [parent, index] = splitLast(locate(anchor.id));
      return [parent, index + 1];
    }
    case "prepend_to":
      return [locate(anchor.id), 0];
    case "append_to": {
      const path = locate(anchor.id);
      return [path, (nodeAtPath(doc, path)!.content ?? []).length];
    }
  }
}

/** Insert `node` at `anchor`, stamping it with `newId` over any id it had. Mutates `doc`. */
export function insertBlock(
  doc: DocNode,
  anchor: Anchor,
  node: DocNode,
  newId: string,
): DocNode {
  if (findPath(doc, newId)) throw OpError.duplicateId(newId);
  const [parent, at] = resolveAnchor(doc, anchor);
  setBlockId(node, newId);
  contentAt(doc, parent).splice(at, 0, node);
  return node;
}

/** Replace the located block, keeping its slot and any id it carried. Mutates `doc`. */
export function replaceBlock(
  doc: DocNode,
  target: Locator,
  node: DocNode,
): DocNode {
  const path = resolveLocator(doc, target);
  if (!path) throw OpError.notFound(locatorToString(target));
  const existing = blockId(nodeAtPath(doc, path)!);
  if (existing !== undefined) setBlockId(node, existing);
  const [parent, index] = splitLast(path);
  contentAt(doc, parent)[index] = node;
  return node;
}

/** Remove the located block and return it. Mutates `doc`. */
export function deleteBlock(doc: DocNode, target: Locator): DocNode {
  const path = resolveLocator(doc, target);
  if (!path) throw OpError.notFound(locatorToString(target));
  const [parent, index] = splitLast(path);
  return contentAt(doc, parent).splice(index, 1)[0]!;
}

/** Move the located block to `anchor` and return it. Mutates `doc`. */
export function moveBlock(
  doc: DocNode,
  target: Locator,
  anchor: Anchor,
): DocNode {
  const src = resolveLocator(doc, target);
  if (!src) throw OpError.notFound(locatorToString(target));
  if (anchor.at !== "start" && anchor.at !== "end") {
    const anchorPath = resolveLocator(doc, anchor.id);
    if (!anchorPath) throw OpError.notFound(locatorToString(anchor.id));
    if (startsWith(anchorPath, src)) throw OpError.selfAnchor();
  }
  // The destination resolves against the original tree so positional paths mean
  // what the caller sees; it is then shifted for the gap the removal leaves.
  const [insParent, insIndexRaw] = resolveAnchor(doc, anchor);
  let insIndex = insIndexRaw;
  const [srcParent, srcIndex] = splitLast(src);
  const node = contentAt(doc, srcParent).splice(srcIndex, 1)[0]!;
  if (startsWith(insParent, srcParent)) {
    const depth = srcParent.length;
    if (insParent.length > depth) {
      if (insParent[depth]! > srcIndex) insParent[depth]! -= 1;
    } else if (insIndex > srcIndex) {
      insIndex -= 1;
    }
  }
  contentAt(doc, insParent).splice(insIndex, 0, node);
  return node;
}

/**
 * Stamp a fresh unique id on every block-group node lacking one, leaving
 * existing ids alone, so ops converge on what the editor would have produced.
 */
export function ensureBlockIds(
  doc: DocNode,
  genId: () => string = generateBlockId,
): void {
  const used = new Set<string>();
  const collect = (node: DocNode) => {
    const id = blockId(node);
    if (id !== undefined) used.add(id);
    node.content?.forEach(collect);
  };
  collect(doc);
  const stamp = (node: DocNode) => {
    for (const child of node.content ?? []) {
      if (isBlock(child) && blockId(child) === undefined) {
        let candidate = genId();
        while (used.has(candidate)) candidate = genId();
        used.add(candidate);
        setBlockId(child, candidate);
      }
      stamp(child);
    }
  };
  stamp(doc);
}

/** Apply one op to `doc`, returning the affected block. Mutates `doc`. */
export function applyOp(
  doc: DocNode,
  op: DocOp,
  genId: () => string = generateBlockId,
): DocNode {
  switch (op.op) {
    case "insert":
      return insertBlock(doc, op.anchor, op.node, genId());
    case "replace":
      return replaceBlock(doc, op.id, op.node);
    case "delete":
      return deleteBlock(doc, op.id);
    case "move":
      return moveBlock(doc, op.id, op.anchor);
  }
}

/**
 * Apply a batch atomically against a working copy: a later op may reference a
 * block an earlier one inserted, and `doc` is untouched if any op throws.
 * Returns the resulting document with ids backfilled on every block.
 */
export function applyAll(
  doc: DocNode,
  ops: DocOp[],
  genId: () => string = generateBlockId,
): DocNode {
  const working = structuredClone(doc);
  for (const op of ops) applyOp(working, op, genId);
  ensureBlockIds(working, genId);
  return working;
}
