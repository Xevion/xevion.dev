import { getSchema } from "@tiptap/core";
import { customAlphabet } from "nanoid";
import type { Mark, MarkType, NodeType, Schema } from "@tiptap/pm/model";
import { codeTokenIds } from "./code-tokens.ts";
import { tiptapExtensions, uniqueIdOptions } from "./extensions.ts";

export interface PmMark {
  type: string;
  attrs?: Record<string, unknown>;
}

export interface PmNode {
  type: string;
  attrs?: Record<string, unknown>;
  content?: PmNode[];
  marks?: PmMark[];
  text?: string;
}

export type ValidationErrorKind =
  | "malformed"
  | "not-a-doc"
  | "unknown-node"
  | "unknown-mark"
  | "disallowed-child"
  | "incomplete-content"
  | "marks-on-non-text"
  | "marks-in-code-block"
  | "mark-not-allowed"
  | "invalid-marks"
  | "empty-text"
  | "text-with-content"
  | "non-text-with-text"
  | "bad-link-scheme"
  | "bad-code-token"
  | "code-too-long";

export interface ValidationError {
  kind: ValidationErrorKind;
  /** Human-readable, with the location appended (see {@link describePath}). */
  message: string;
  /** Child indices from the root down to the offending node; empty for the root itself. */
  path: number[];
}

export type ValidationResult =
  | { ok: true; doc: PmNode }
  | ({ ok: false } & ValidationError);

export type PrepareResult =
  | { ok: true; doc: PmNode | null }
  | ({ ok: false } & ValidationError);

export const ID_ATTR = "id";
export const MAX_CODE_TEXT_CHARS = 4096;

// Depth beyond which the walk would risk the call stack; real documents sit far below it.
const MAX_DEPTH = 128;

const LINK_SCHEMES = ["http://", "https://", "mailto:"];

const schema: Schema = getSchema(tiptapExtensions);

const ID_TYPES: ReadonlySet<string> = new Set(uniqueIdOptions?.types ?? []);

/** Allowed node type names, derived from the editor schema. */
export const nodeTypeNames: readonly string[] = Object.keys(schema.nodes);

/** Allowed mark type names, derived from the editor schema. */
export const markTypeNames: readonly string[] = Object.keys(schema.marks);

/** Node types that carry a stable block id. */
export const blockIdTypes: ReadonlySet<string> = ID_TYPES;

const defaultBlockId = customAlphabet(
  "abcdefghijklmnopqrstuvwxyz0123456789",
  8,
);

/** Mints an 8-character `[a-z0-9]` block id. */
export function generateBlockId(): string {
  return defaultBlockId();
}

/** Formats a node path like `.3.0` (empty path is the document root). */
export function describePath(path: readonly number[]): string {
  return path.length === 0 ? "the document root" : `.${path.join(".")}`;
}

class Failure extends Error {
  constructor(
    readonly kind: ValidationErrorKind,
    detail: string,
    readonly path: number[],
  ) {
    super(detail);
  }
}

function fail(
  kind: ValidationErrorKind,
  detail: string,
  path: number[],
): never {
  throw new Failure(kind, detail, path);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function nodeType(name: string): NodeType | undefined {
  return Object.hasOwn(schema.nodes, name) ? schema.nodes[name] : undefined;
}

function markType(name: string): MarkType | undefined {
  return Object.hasOwn(schema.marks, name) ? schema.marks[name] : undefined;
}

function readAttrs(
  value: unknown,
  path: number[],
  what: string,
): Record<string, unknown> | undefined {
  if (value === undefined || value === null) return undefined;
  if (!isRecord(value))
    fail("malformed", `${what} attrs must be an object`, path);
  return Object.keys(value).length === 0 ? undefined : { ...value };
}

function checkMark(
  raw: unknown,
  text: string,
  path: number[],
): { json: PmMark; mark: Mark } {
  if (!isRecord(raw) || typeof raw.type !== "string")
    fail("malformed", "a mark must be an object with a string type", path);
  const type = markType(raw.type);
  if (!type) fail("unknown-mark", `unknown mark type "${raw.type}"`, path);
  const attrs = readAttrs(raw.attrs, path, "mark");

  if (raw.type === "link") {
    const href = typeof attrs?.href === "string" ? attrs.href : "";
    const lower = href.toLowerCase();
    if (!LINK_SCHEMES.some((scheme) => lower.startsWith(scheme)))
      fail(
        "bad-link-scheme",
        `link uses a disallowed URL scheme: "${href}"`,
        path,
      );
  }

  if (raw.type === "code") {
    const token = attrs?.token;
    if (token !== undefined && token !== null) {
      if (typeof token !== "string" || !codeTokenIds.has(token)) {
        fail(
          "bad-code-token",
          `unknown inline-code token kind: "${typeof token === "string" ? token : JSON.stringify(token)}"`,
          path,
        );
      }
    }
    const length = [...text].length;
    if (length > MAX_CODE_TEXT_CHARS)
      fail(
        "code-too-long",
        `inline-code text is too long: ${length} characters (max ${MAX_CODE_TEXT_CHARS})`,
        path,
      );
  }

  let mark: Mark;
  try {
    mark = type.create(attrs);
  } catch (cause) {
    fail("malformed", `invalid "${raw.type}" mark: ${String(cause)}`, path);
  }
  return { json: attrs ? { type: raw.type, attrs } : { type: raw.type }, mark };
}

interface Checked {
  json: PmNode;
  type: NodeType;
  marks: readonly Mark[];
}

function checkNode(raw: unknown, path: number[]): Checked {
  if (path.length > MAX_DEPTH)
    fail("malformed", "document is nested too deeply", path);
  if (!isRecord(raw) || typeof raw.type !== "string")
    fail("malformed", "a node must be an object with a string type", path);

  const type = nodeType(raw.type);
  if (!type) fail("unknown-node", `unknown node type "${raw.type}"`, path);

  const rawContent = raw.content ?? undefined;
  if (rawContent !== undefined && !Array.isArray(rawContent))
    fail("malformed", "node content must be an array", path);
  const rawMarks = raw.marks ?? undefined;
  if (rawMarks !== undefined && !Array.isArray(rawMarks))
    fail("malformed", "node marks must be an array", path);
  const rawText = raw.text ?? undefined;
  if (rawText !== undefined && typeof rawText !== "string")
    fail("malformed", "node text must be a string", path);
  const attrs = readAttrs(raw.attrs, path, "node");

  const isText = raw.type === "text";
  if (isText) {
    if (rawText === undefined || rawText === "")
      fail("empty-text", "text nodes cannot be empty", path);
    if (rawContent !== undefined && rawContent.length > 0)
      fail("text-with-content", "a text node cannot have child content", path);
  } else if (rawText !== undefined) {
    fail(
      "non-text-with-text",
      `non-text node "${raw.type}" cannot carry a text field`,
      path,
    );
  }

  if (rawMarks !== undefined && rawMarks.length > 0 && !isText)
    fail(
      "marks-on-non-text",
      `marks are only allowed on text nodes, found on "${raw.type}"`,
      path,
    );

  const checkedMarks = (rawMarks ?? []).map((rawMark) =>
    checkMark(rawMark, rawText ?? "", path),
  );
  // addToSet orders by schema rank, which is how ProseMirror stores marks, so authored order is
  // canonicalized instead of rejected; a short result means duplicates or excluded combinations.
  let marks: readonly Mark[] = [];
  for (const { mark } of checkedMarks) marks = mark.addToSet(marks);
  if (marks.length !== checkedMarks.length)
    fail(
      "invalid-marks",
      "marks are duplicated or cannot be combined on one text node",
      path,
    );
  const markJson = marks.map(
    (mark) => checkedMarks.find((checked) => checked.mark === mark)!.json,
  );

  const content: PmNode[] = [];
  let match = type.contentMatch;
  for (const [index, rawChild] of (rawContent ?? []).entries()) {
    const childPath = [...path, index];
    const child = checkNode(rawChild, childPath);
    const next = match.matchType(child.type);
    if (!next)
      fail(
        "disallowed-child",
        `node "${child.type.name}" is not allowed inside "${raw.type}"`,
        childPath,
      );
    match = next;
    if (child.marks.length > 0 && !type.allowsMarks(child.marks)) {
      if (type.spec.code)
        fail(
          "marks-in-code-block",
          "code blocks cannot contain formatting marks",
          childPath,
        );
      fail(
        "mark-not-allowed",
        `formatting marks are not allowed inside "${raw.type}"`,
        childPath,
      );
    }
    content.push(child.json);
  }
  if (path.length > 0 && !match.validEnd)
    fail(
      "incomplete-content",
      `node "${raw.type}" is missing required content`,
      path,
    );

  const json: PmNode = { type: raw.type };
  if (attrs) json.attrs = attrs;
  if (content.length > 0) json.content = content;
  if (markJson.length > 0) json.marks = markJson;
  if (rawText !== undefined) json.text = rawText;
  return { json, type, marks };
}

function toError(failure: Failure): ValidationError {
  const detail =
    failure.kind === "malformed"
      ? `detail content is not a valid document: ${failure.message}`
      : failure.message;
  return {
    kind: failure.kind,
    message: `${detail} (at ${describePath(failure.path)})`,
    path: failure.path,
  };
}

/**
 * Validates a detail document against the editor schema plus the write-path rules the schema
 * cannot express (link schemes, code-mark token vocabulary and length cap, no marks in code
 * blocks). Returns the canonical document (unknown fields dropped, empty attrs/marks/content
 * omitted); never throws.
 */
export function validateDoc(value: unknown): ValidationResult {
  try {
    if (!isRecord(value) || typeof value.type !== "string")
      fail("malformed", "the root must be an object with a string type", []);
    if (value.type !== "doc")
      fail(
        "not-a-doc",
        `document root must be a "doc" node, found "${value.type}"`,
        [],
      );

    const { json } = checkNode(value, []);
    // An empty doc is valid here even though the schema requires one block; it stores as null.
    if (json.content) {
      try {
        schema.nodeFromJSON(json).check();
      } catch (cause) {
        fail("malformed", String(cause), []);
      }
    }
    return { ok: true, doc: json };
  } catch (caught) {
    if (caught instanceof Failure) return { ok: false, ...toError(caught) };
    throw caught;
  }
}

function blockId(node: PmNode): string | undefined {
  const id = node.attrs?.[ID_ATTR];
  return typeof id === "string" && id !== "" ? id : undefined;
}

function collectIds(node: PmNode, used: Set<string>): void {
  const id = blockId(node);
  if (id) used.add(id);
  for (const child of node.content ?? []) collectIds(child, used);
}

function stamp(node: PmNode, used: Set<string>, genId: () => string): PmNode {
  if (!node.content) return node;
  return {
    ...node,
    content: node.content.map((child) => {
      let next = child;
      if (ID_TYPES.has(child.type) && !blockId(child)) {
        let id = genId();
        while (used.has(id)) id = genId();
        used.add(id);
        next = { ...child, attrs: { ...child.attrs, [ID_ATTR]: id } };
      }
      return stamp(next, used, genId);
    }),
  };
}

/**
 * Returns a copy of `doc` with a unique id stamped on every block node lacking one, leaving
 * existing ids alone. The root and inline nodes are never stamped.
 */
export function ensureBlockIds(
  doc: PmNode,
  genId: () => string = generateBlockId,
): PmNode {
  const used = new Set<string>();
  collectIds(doc, used);
  return stamp(doc, used, genId);
}

/** The value to persist for `doc`: null for a document with no blocks. */
export function toStored(doc: PmNode): PmNode | null {
  return doc.content && doc.content.length > 0 ? doc : null;
}

/** Validates, stamps missing block ids, and reduces an empty document to null. */
export function prepareDoc(
  value: unknown,
  genId: () => string = generateBlockId,
): PrepareResult {
  const result = validateDoc(value);
  if (!result.ok) return result;
  return { ok: true, doc: toStored(ensureBlockIds(result.doc, genId)) };
}
