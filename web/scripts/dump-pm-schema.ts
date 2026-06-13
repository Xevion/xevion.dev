/**
 * Emits `src/pm_schema.generated.json` — the canonical node + mark name set the
 * TipTap document schema permits. The Rust `pm` module's `schema_sync` test
 * `include_str!`s that file and asserts its NODES/MARKS allow-list matches, so
 * the Rust validator and the editor can never silently diverge.
 *
 * Also asserts the server renderer (`tiptapExtensions`) and the editor
 * (`editorExtensions`) expose an identical node/mark set — the two arrays are
 * hand-maintained and only meant to differ on the codeBlock implementation.
 *
 * Run via `bun run --cwd web dump-schema`. `tempo check` regenerates it
 * automatically when the extension files change and fails on a dirty diff.
 */
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { getSchema } from "@tiptap/core";
import type { Extensions } from "@tiptap/core";
import { tiptapExtensions } from "../src/lib/tiptap/extensions";
import { editorExtensions } from "../src/lib/tiptap/extensions.editor";

interface SchemaNames {
  nodes: string[];
  marks: string[];
}

function schemaNames(extensions: Extensions): SchemaNames {
  const schema = getSchema(extensions);
  return {
    nodes: Object.keys(schema.nodes).sort(),
    marks: Object.keys(schema.marks).sort(),
  };
}

/** Throw (non-zero exit) if two name sets diverge, listing the offenders. */
function assertSameSet(kind: string, server: string[], editor: string[]): void {
  const serverSet = new Set(server);
  const editorSet = new Set(editor);
  const serverOnly = server.filter((name) => !editorSet.has(name));
  const editorOnly = editor.filter((name) => !serverSet.has(name));
  if (serverOnly.length || editorOnly.length) {
    throw new Error(
      `${kind} differ between server and editor extensions:\n` +
        `  server-only: ${serverOnly.join(", ") || "(none)"}\n` +
        `  editor-only: ${editorOnly.join(", ") || "(none)"}`,
    );
  }
}

const server = schemaNames(tiptapExtensions);
const editor = schemaNames(editorExtensions);

assertSameSet("node types", server.nodes, editor.nodes);
assertSameSet("mark types", server.marks, editor.marks);

const outPath = join(import.meta.dir, "../../src/pm_schema.generated.json");
writeFileSync(outPath, `${JSON.stringify(server, null, 2)}\n`);

console.log(
  `Wrote ${outPath}\n  nodes: ${server.nodes.join(", ")}\n  marks: ${server.marks.join(", ")}`,
);
