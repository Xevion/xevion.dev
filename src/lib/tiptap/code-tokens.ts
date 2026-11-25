/**
 * Author-declared inline-code token kinds: the `{:.kind}` authoring suffix and
 * the `code` mark's `token` attr. A closed vocabulary: the renderer paints each
 * via a `.tk-<id>` class, the editor offers them in its inline-code picker, and
 * the write-path validator (validate.ts) rejects a `token` outside this set.
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
