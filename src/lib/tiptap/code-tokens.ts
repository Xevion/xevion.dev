/**
 * Author-declared inline-code token kinds — the `{:.kind}` authoring suffix and
 * the `code` mark's `token` attr. A closed vocabulary mirrored from Rust's
 * `CODE_TOKEN_KINDS` (src/pm.rs): the SSR renderer paints each via a `.tk-<id>`
 * class and the editor offers them in its inline-code picker. Keep in sync with
 * pm.rs — the model rejects a `token` outside this set on the write path.
 */
export interface CodeToken {
  id: string;
  label: string;
}

export const codeTokens: CodeToken[] = [
  { id: "keyword", label: "Keyword" },
  { id: "fn", label: "Function" },
  { id: "type", label: "Type" },
  { id: "string", label: "String" },
  { id: "number", label: "Number" },
  { id: "const", label: "Constant" },
  { id: "var", label: "Variable" },
  { id: "flag", label: "Flag" },
  { id: "comment", label: "Comment" },
];

/** Fast membership check used by the renderer to reject unknown kinds. */
export const codeTokenIds: ReadonlySet<string> = new Set(
  codeTokens.map((token) => token.id),
);
