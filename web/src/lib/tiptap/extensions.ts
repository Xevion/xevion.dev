import StarterKit from "@tiptap/starter-kit";
import type { Extensions } from "@tiptap/core";

type StarterKitOptions = Parameters<typeof StarterKit.configure>[0];

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
];
