import StarterKit from "@tiptap/starter-kit";
import { UniqueID } from "@tiptap/extension-unique-id";
import { customAlphabet } from "nanoid";
import type { Extensions } from "@tiptap/core";

type StarterKitOptions = Parameters<typeof StarterKit.configure>[0];
type UniqueIdOptions = Parameters<typeof UniqueID.configure>[0];

/**
 * Document-schema configuration shared by the server renderer and the editor.
 * Extracting it keeps the two extension arrays from drifting on everything
 * except the code-block implementation (see {@link editorExtensions}).
 */
export const starterKitOptions: StarterKitOptions = {
  heading: { levels: [2, 3, 4] },
  link: {
    openOnClick: false,
    HTMLAttributes: { rel: "noopener noreferrer nofollow", target: "_blank" },
  },
};

/**
 * Mints an 8-character lowercase-alphanumeric id, kept byte-for-byte compatible
 * with Rust's `generate_block_id` (src/pm.rs): the same 36-char `[a-z0-9]`
 * alphabet and length, so a block's id reads identically whether it was minted
 * in the editor or stamped server-side.
 */
const generateBlockId = customAlphabet(
  "abcdefghijklmnopqrstuvwxyz0123456789",
  8,
);

/**
 * Stamps a stable `id` attr onto every block-level node. Shared by both
 * extension arrays so authored ids survive the round-trip to the server renderer
 * and back. The attribute name defaults to `id`, matching `pm::ID_ATTR`; Rust's
 * `/content` ops address top-level blocks by this id, and ids on nested nodes
 * (a paragraph inside a list item, say) ride along as inert, harmless data.
 */
export const uniqueIdOptions: UniqueIdOptions = {
  types: [
    "paragraph",
    "heading",
    "blockquote",
    "codeBlock",
    "horizontalRule",
    "bulletList",
    "orderedList",
    "listItem",
  ],
  generateID: () => generateBlockId(),
};

/**
 * Canonical schema used by the server-side renderer (render.server.ts). The
 * single source of truth for what nodes/attrs a detail document may contain.
 * StarterKit's default `codeBlock` supplies the `codeBlock` node (with a
 * `language` attr); the renderer overrides its HTML output with Shiki syntax
 * highlighting via `nodeMapping`. The editor swaps that same node for a
 * lowlight-backed variant with an identical schema — see {@link editorExtensions}.
 *
 * Any new custom node MUST be added here (and to the editor array) so both
 * contexts see it.
 */
export const tiptapExtensions: Extensions = [
  StarterKit.configure(starterKitOptions),
  UniqueID.configure(uniqueIdOptions),
];
