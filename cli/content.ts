import { readFileSync } from "node:fs";
import { text as readStream } from "node:stream/consumers";
import { authedClient, loadConfig } from "./auth.ts";
import type { ApiClient } from "./client.ts";
import { CliError } from "./errors.ts";
import type { CommandNode, Context } from "./framework.ts";
import * as output from "./output.ts";
import { MarkdownError, markdownToBlocks } from "#lib/tiptap/markdown.ts";
import {
  OpError,
  applyAll,
  docAt,
  docFromStored,
  insertSequence,
  parseAnchor,
  parseLocator,
  replaceSequence,
  type DocNode,
  type DocOp,
  type Locator,
} from "#lib/tiptap/ops.ts";

const seg = encodeURIComponent;

/**
 * Read all of stdin. The `-` sentinel on every authoring input routes here so a
 * document can be piped in rather than crammed into a shell-quoted argument.
 */
async function readStdin(): Promise<string> {
  try {
    return await readStream(process.stdin);
  } catch (e) {
    throw CliError.io("<stdin>", e);
  }
}

async function orStdin(value: string): Promise<string> {
  return value === "-" ? readStdin() : value;
}

function asInvalid<T>(fn: () => T): T {
  try {
    return fn();
  } catch (e) {
    if (e instanceof CliError) throw e;
    if (e instanceof Error) throw CliError.invalid(e.message);
    throw e;
  }
}

function locator(raw: string): Locator {
  return asInvalid(() => parseLocator(raw));
}

interface Fetched {
  doc: DocNode;
  etag?: string;
}

async function fetchDoc(
  client: ApiClient,
  reference: string,
): Promise<Fetched> {
  const res = await client.send<unknown>(
    "GET",
    `/api/projects/${seg(reference)}/content`,
  );
  return {
    doc: docFromStored(res.data),
    etag: res.headers.get("etag") ?? undefined,
  };
}

async function putDoc(
  client: ApiClient,
  reference: string,
  doc: DocNode,
  etag: string | undefined,
): Promise<DocNode> {
  try {
    const res = await client.send<unknown>(
      "PUT",
      `/api/projects/${seg(reference)}/content`,
      { body: doc, headers: etag ? { "if-match": etag } : undefined },
    );
    return docFromStored(res.data);
  } catch (e) {
    if (e instanceof CliError && e.status === 412) {
      throw new CliError(
        "http",
        "the project content was edited by someone else since it was fetched; nothing was saved",
        {
          code: e.code ?? "PRECONDITION_FAILED",
          help: "re-run the command to apply it to the latest content",
          status: 412,
        },
      );
    }
    throw e;
  }
}

/** Apply ops locally to the fetched document, then write it back conditionally. */
async function applyOps(
  client: ApiClient,
  reference: string,
  ops: DocOp[],
): Promise<DocNode> {
  const { doc, etag } = await fetchDoc(client, reference);
  const next = asInvalid(() => {
    try {
      return applyAll(doc, ops);
    } catch (e) {
      if (e instanceof OpError) throw CliError.invalid(e.message);
      throw e;
    }
  });
  return putDoc(client, reference, next, etag);
}

function report(doc: DocNode, ctx: Context, quiet: boolean, msg: string): void {
  if (ctx.json) {
    output.printJson(doc);
    return;
  }
  output.success(msg);
  if (!quiet) output.printBlocks(doc);
}

/** Resolve an insert/replace body into blocks from `--md` or a raw `--node`. */
async function bodyBlocks(
  md: string | undefined,
  node: string | undefined,
): Promise<DocNode[]> {
  if (md !== undefined && node !== undefined) {
    throw CliError.invalid("the argument '--md' cannot be used with '--node'");
  }
  if (md !== undefined) {
    const source = await orStdin(md);
    try {
      return markdownToBlocks(source);
    } catch (e) {
      if (e instanceof MarkdownError) {
        throw CliError.invalid(`--md: ${e.message}`);
      }
      throw e;
    }
  }
  if (node !== undefined) {
    const source = await orStdin(node);
    let parsed: unknown;
    try {
      parsed = JSON.parse(source);
    } catch (e) {
      throw CliError.invalid(
        `--node is not a valid ProseMirror node: ${e instanceof Error ? e.message : String(e)}`,
      );
    }
    if (
      typeof parsed !== "object" ||
      parsed === null ||
      typeof (parsed as { type?: unknown }).type !== "string"
    ) {
      throw CliError.invalid(
        '--node is not a valid ProseMirror node: expected an object with a string "type"',
      );
    }
    return [parsed as DocNode];
  }
  throw CliError.invalid(
    "the following required arguments were not provided: --md <MD> or --node <NODE>",
  );
}

const QUIET_FLAG = {
  type: Boolean,
  alias: "q",
  description: "Print only the confirmation line, not the re-rendered document",
} as const;

const AT_DOC =
  "start | end | before:<loc> | after:<loc> | prepend:<loc> | append:<loc>";

function client(ctx: Context): ApiClient {
  return authedClient(loadConfig(ctx), ctx.api);
}

export const contentCommand: CommandNode = {
  name: "content",
  description: "Edit detail-page block content",
  children: [
    {
      name: "list",
      description: "List blocks (id, type, preview)",
      parameters: ["<ref>"],
      run: async ({ _ }, ctx) => {
        const { doc } = await fetchDoc(client(ctx), String(_.ref));
        if (ctx.json) output.printJson(doc);
        else output.printBlocks(doc);
      },
    },
    {
      name: "get",
      description:
        "Print the document as JSON, or a single block by locator (a path like .3 or .3.0, or a block id)",
      parameters: ["<ref>", "[locator]"],
      run: async ({ _ }, ctx) => {
        const { doc } = await fetchDoc(client(ctx), String(_.ref));
        if (typeof _.locator !== "string") {
          output.printJson(doc);
          return;
        }
        const node = docAt(doc, locator(_.locator));
        if (!node) throw CliError.invalid(`no block at "${_.locator}"`);
        output.printJson(node);
      },
    },
    {
      name: "insert",
      description:
        "Insert new block(s), authored as Markdown (--md) or a raw node (--node)",
      parameters: ["<ref>"],
      flags: {
        at: {
          type: String,
          description: `Position: ${AT_DOC} [default: end]`,
          placeholder: "<pos>",
        },
        md: {
          type: String,
          description:
            "Block(s) as Markdown, e.g. '## Heading' or '- a bullet'; '-' reads stdin",
          placeholder: "<md>",
        },
        node: {
          type: String,
          description:
            "Raw ProseMirror node JSON (escape hatch); '-' reads stdin (pipe to dodge quoting)",
          placeholder: "<json>",
        },
        quiet: QUIET_FLAG,
      },
      run: async ({ _, flags }, ctx) => {
        const anchor = asInvalid(() => parseAnchor(flags.at ?? "end"));
        const blocks = await bodyBlocks(flags.md, flags.node);
        const ops = insertSequence(anchor, blocks);
        const doc = await applyOps(client(ctx), String(_.ref), ops);
        report(
          doc,
          ctx,
          Boolean(flags.quiet),
          `Inserted ${blocks.length} block(s)`,
        );
      },
    },
    {
      name: "replace",
      description:
        "Replace a block, authored as Markdown (--md) or a raw node (--node)",
      parameters: ["<ref>", "<locator>"],
      flags: {
        md: {
          type: String,
          description:
            "Replacement as Markdown; multiple blocks replace the target then follow it; '-' reads stdin",
          placeholder: "<md>",
        },
        node: {
          type: String,
          description:
            "Raw ProseMirror node JSON (escape hatch), keeps the target's id; '-' reads stdin",
          placeholder: "<json>",
        },
        quiet: QUIET_FLAG,
      },
      run: async ({ _, flags }, ctx) => {
        const target = locator(String(_.locator));
        const blocks = await bodyBlocks(flags.md, flags.node);
        const ops = replaceSequence(target, blocks);
        const doc = await applyOps(client(ctx), String(_.ref), ops);
        report(doc, ctx, Boolean(flags.quiet), `Replaced block ${_.locator}`);
      },
    },
    {
      name: "rm",
      description:
        "Remove a block (locator: a path like .3 or .3.0, or a block id)",
      parameters: ["<ref>", "<locator>"],
      flags: { quiet: QUIET_FLAG },
      run: async ({ _, flags }, ctx) => {
        const op: DocOp = { op: "delete", id: locator(String(_.locator)) };
        const doc = await applyOps(client(ctx), String(_.ref), [op]);
        report(doc, ctx, Boolean(flags.quiet), `Removed block ${_.locator}`);
      },
    },
    {
      name: "move",
      description: "Move a block to a new position",
      parameters: ["<ref>", "<locator>"],
      flags: {
        at: {
          type: String,
          description: `Position: ${AT_DOC} (required)`,
          placeholder: "<pos>",
        },
        quiet: QUIET_FLAG,
      },
      run: async ({ _, flags }, ctx) => {
        if (typeof flags.at !== "string") {
          throw CliError.invalid(
            "the following required arguments were not provided: --at <pos>",
          );
        }
        const op: DocOp = {
          op: "move",
          id: locator(String(_.locator)),
          anchor: asInvalid(() => parseAnchor(flags.at)),
        };
        const doc = await applyOps(client(ctx), String(_.ref), [op]);
        report(doc, ctx, Boolean(flags.quiet), `Moved block ${_.locator}`);
      },
    },
    {
      name: "set",
      description: "Replace the entire document from a JSON file (or stdin)",
      parameters: ["<ref>"],
      flags: {
        file: {
          type: String,
          description:
            "Path to a JSON file holding the full ProseMirror document; '-' reads stdin",
          placeholder: "<file>",
        },
        quiet: QUIET_FLAG,
      },
      run: async ({ _, flags }, ctx) => {
        if (typeof flags.file !== "string") {
          throw CliError.invalid(
            "the following required arguments were not provided: --file <file>",
          );
        }
        const label = flags.file === "-" ? "<stdin>" : flags.file;
        let raw: string;
        if (flags.file === "-") {
          raw = await readStdin();
        } else {
          try {
            raw = readFileSync(flags.file, "utf8");
          } catch (e) {
            throw CliError.io(flags.file, e);
          }
        }
        let value: DocNode;
        try {
          value = JSON.parse(raw) as DocNode;
        } catch (e) {
          throw CliError.json(label, e);
        }
        const api = client(ctx);
        // The PUT is conditional on the latest ETag, so a replace never silently
        // overwrites an edit made after this command started.
        const { etag } = await fetchDoc(api, String(_.ref));
        const doc = await putDoc(api, String(_.ref), value, etag);
        report(doc, ctx, Boolean(flags.quiet), "Replaced document");
      },
    },
  ],
};
