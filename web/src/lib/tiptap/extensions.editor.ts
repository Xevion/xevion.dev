import StarterKit from "@tiptap/starter-kit";
import { CodeBlockLowlight } from "@tiptap/extension-code-block-lowlight";
import { UniqueID } from "@tiptap/extension-unique-id";
import { createLowlight, common } from "lowlight";
import type { Extensions } from "@tiptap/core";
import {
  Figure,
  Gloss,
  Sidenote,
  starterKitOptions,
  uniqueIdOptions,
} from "./extensions";

const lowlight = createLowlight(common);

/**
 * Editor schema. Identical document schema to {@link tiptapExtensions}, but
 * swaps StarterKit's plain `codeBlock` for CodeBlockLowlight so the admin sees
 * in-editor syntax colors. Both define the same `codeBlock` node with the same
 * `language` attribute, so authored JSON round-trips to the server renderer
 * unchanged — the server simply re-highlights with Shiki instead of lowlight.
 *
 * The editor highlights with lowlight (highlight.js `common` grammars); which
 * picker languages it covers vs. only Shiki does is noted in {@link codeLanguages}.
 */
export const editorExtensions: Extensions = [
  StarterKit.configure({ ...starterKitOptions, codeBlock: false }),
  CodeBlockLowlight.configure({ lowlight, defaultLanguage: null }),
  UniqueID.configure(uniqueIdOptions),
  Figure,
  Gloss,
  Sidenote,
];
