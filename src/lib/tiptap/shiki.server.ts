import {
  createHighlighterCore,
  type HighlighterCore,
  type LanguageInput,
} from "shiki/core";
import { createJavaScriptRegexEngine } from "shiki/engine/javascript";
import { codeLanguages } from "./languages";

/**
 * Shiki grammar loaders keyed by the language id used in {@link codeLanguages}.
 * This is a registry, not the loaded set — the set that actually loads is
 * derived from `codeLanguages` below, so it can never offer a language the
 * picker doesn't, nor (silently) lack one the picker does.
 */
const grammarLoaders: Record<string, LanguageInput> = {
  rust: () => import("@shikijs/langs/rust"),
  typescript: () => import("@shikijs/langs/typescript"),
  javascript: () => import("@shikijs/langs/javascript"),
  svelte: () => import("@shikijs/langs/svelte"),
  html: () => import("@shikijs/langs/html"),
  css: () => import("@shikijs/langs/css"),
  json: () => import("@shikijs/langs/json"),
  yaml: () => import("@shikijs/langs/yaml"),
  toml: () => import("@shikijs/langs/toml"),
  sql: () => import("@shikijs/langs/sql"),
  bash: () => import("@shikijs/langs/bash"),
  python: () => import("@shikijs/langs/python"),
};

/** Picker ids that need a Shiki grammar — everything except plain text. */
const highlightedLangIds = codeLanguages
  .map((lang) => lang.id)
  .filter((id) => id !== "text");

// Fail fast if the picker offers a language with no grammar loader: that block
// would silently render as plain text. Throwing at module load surfaces the
// drift the moment a language is added to codeLanguages without a loader here.
const missingLoaders = highlightedLangIds.filter(
  (id) => !(id in grammarLoaders),
);
if (missingLoaders.length > 0) {
  throw new Error(
    `shiki.server: no grammar loader for code language(s): ${missingLoaders.join(", ")}`,
  );
}

/**
 * Server-side syntax highlighter for rendered detail content. Built on Shiki's
 * fine-grained core with a fixed theme set (not the full ~6 MB bundle) and the
 * WASM-free JavaScript regex engine, which suits the Bun SSR runtime. The
 * language set is derived from {@link codeLanguages}, so it tracks the editor's
 * picker automatically.
 *
 * `createHighlighterCore` is async (it loads grammars), but the resolved
 * instance's `codeToHtml` is synchronous — which is what lets us call it inside
 * the static renderer's synchronous `nodeMapping`. We memoize the promise so the
 * grammars load once per process.
 */
let highlighterPromise: Promise<HighlighterCore> | null = null;

export function getHighlighter(): Promise<HighlighterCore> {
  if (!highlighterPromise) {
    highlighterPromise = createHighlighterCore({
      engine: createJavaScriptRegexEngine(),
      themes: [
        import("@shikijs/themes/github-light"),
        import("@shikijs/themes/github-dark"),
      ],
      langs: highlightedLangIds.map((id) => grammarLoaders[id]),
    }).catch((err) => {
      // Don't memoize a transient grammar-load failure — clearing the cache lets
      // the next request retry instead of breaking detail rendering until restart.
      highlighterPromise = null;
      throw err;
    });
  }
  return highlighterPromise;
}
