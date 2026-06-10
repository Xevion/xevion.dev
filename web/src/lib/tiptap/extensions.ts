import StarterKit from "@tiptap/starter-kit";
import type { Extensions } from "@tiptap/core";

/**
 * The single source of truth for the editor schema. Shared by the admin editor
 * (svelte-tiptap) and the server-side renderer (@tiptap/static-renderer) so the
 * authored document and the rendered HTML can never drift out of sync.
 *
 * Slice 1: standard rich text only. Custom block nodes (code/gallery/embeds)
 * land in XEV-980 and must be appended here so both contexts pick them up.
 */
export const tiptapExtensions: Extensions = [
  StarterKit.configure({
    heading: { levels: [2, 3, 4] },
    link: {
      openOnClick: false,
      HTMLAttributes: { rel: "noopener noreferrer nofollow", target: "_blank" },
    },
  }),
];
