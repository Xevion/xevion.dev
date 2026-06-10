/**
 * Single source of truth for the code-block languages offered in the editor's
 * picker. The `id` is a Shiki language identifier; the server renderer
 * (shiki.server.ts) derives its loaded grammar set from this list — and asserts
 * a loader exists for every non-"text" id at boot — so what it highlights can
 * never drift from what the picker offers. The editor's lowlight engine
 * highlights whatever subset it knows (svelte/toml have no highlight.js grammar,
 * so they render plain in-editor but still highlight server-side via Shiki).
 * Unknown ids fall back to plain text.
 */
export interface CodeLanguage {
  id: string;
  label: string;
}

export const codeLanguages: CodeLanguage[] = [
  { id: "text", label: "Plain text" },
  { id: "rust", label: "Rust" },
  { id: "typescript", label: "TypeScript" },
  { id: "javascript", label: "JavaScript" },
  { id: "svelte", label: "Svelte" },
  { id: "html", label: "HTML" },
  { id: "css", label: "CSS" },
  { id: "json", label: "JSON" },
  { id: "yaml", label: "YAML" },
  { id: "toml", label: "TOML" },
  { id: "sql", label: "SQL" },
  { id: "bash", label: "Shell" },
  { id: "python", label: "Python" },
];
