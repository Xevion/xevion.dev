import StarterKit from "@tiptap/starter-kit";
import { UniqueID } from "@tiptap/extension-unique-id";
import { Node, Mark } from "@tiptap/core";
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
    "figure",
    "sidenote",
    "callout",
    "details",
  ],
  generateID: () => generateBlockId(),
};

/**
 * String attr that round-trips through a `data-*` HTML attribute, so a node with
 * no editor node-view still preserves its data when the admin editor re-saves
 * the document (TipTap serializes via `renderHTML`, re-parses via `parseHTML`).
 */
function dataAttr(name: string, fallback: string | null = null) {
  return {
    default: fallback,
    parseHTML: (el: HTMLElement) => el.getAttribute(`data-${name}`) ?? fallback,
    renderHTML: (attrs: Record<string, unknown>) =>
      attrs[name] == null ? {} : { [`data-${name}`]: String(attrs[name]) },
  };
}

/**
 * Inline media block. An atom carrying `src`/`alt`/`caption`/`kind` in attrs; the
 * server renderer ({@link file://./render.server.ts}) emits the real
 * `<figure><img|video><figcaption>` markup via `nodeMapping`. The editor keeps
 * the data on a `<figure data-figure>` shell so it survives a GUI round-trip even
 * before a dedicated node-view exists. Authored via the CLI's `content` `--node`.
 */
export const Figure = Node.create({
  name: "figure",
  group: "block",
  atom: true,
  draggable: true,
  addAttributes() {
    return {
      src: dataAttr("src"),
      alt: dataAttr("alt"),
      caption: dataAttr("caption"),
      kind: dataAttr("kind", "image"),
    };
  },
  parseHTML() {
    return [{ tag: "figure[data-figure]" }];
  },
  renderHTML({ HTMLAttributes }) {
    return ["figure", { "data-figure": "", ...HTMLAttributes }];
  },
});

/**
 * Inline gloss: a dotted-underlined span whose `note` shows as a small popover on
 * hover/focus (the "what is a transpose pass?" annotation). A mark, like `link`,
 * carrying its text in a `data-note` attribute that the prose CSS reads via
 * `attr()`. Both contexts render through this same `renderHTML`.
 */
export const Gloss = Mark.create({
  name: "gloss",
  inclusive: false,
  addAttributes() {
    return {
      note: {
        default: null,
        parseHTML: (el: HTMLElement) => el.getAttribute("data-note"),
        renderHTML: (attrs: Record<string, unknown>) =>
          attrs.note == null ? {} : { "data-note": String(attrs.note) },
      },
    };
  },
  parseHTML() {
    return [{ tag: "span[data-note]" }];
  },
  renderHTML({ HTMLAttributes }) {
    return ["span", { class: "gloss", tabindex: "0", ...HTMLAttributes }, 0];
  },
});

/**
 * Margin/side note: block content set apart from the main reading column (floated
 * aside on wide screens, a quiet full-width note on mobile). Distinct from the
 * louder typed callouts. Renders as `<aside data-sidenote>` in both contexts.
 */
export const Sidenote = Node.create({
  name: "sidenote",
  group: "block",
  content: "block+",
  defining: true,
  parseHTML() {
    return [{ tag: "aside[data-sidenote]" }];
  },
  renderHTML() {
    return ["aside", { "data-sidenote": "", class: "rd-sidenote" }, 0];
  },
});

/**
 * Typed admonition (note / tip / warning). Block content with a `variant` attr;
 * the prose CSS draws the per-variant color and a masked icon off
 * `[data-variant]`, so the rendered HTML carries no inline SVG.
 */
export const Callout = Node.create({
  name: "callout",
  group: "block",
  content: "block+",
  defining: true,
  addAttributes() {
    return {
      variant: {
        default: "note",
        parseHTML: (el: HTMLElement) =>
          el.getAttribute("data-variant") ?? "note",
        renderHTML: (attrs: Record<string, unknown>) => ({
          "data-variant": String(attrs.variant ?? "note"),
        }),
      },
    };
  },
  parseHTML() {
    return [{ tag: "aside[data-callout]" }];
  },
  renderHTML({ HTMLAttributes }) {
    return [
      "aside",
      { "data-callout": "", class: "rd-callout", ...HTMLAttributes },
      0,
    ];
  },
});

/**
 * Collapsible disclosure. `summary` is the always-visible toggle label (rendered
 * as a static `<summary>` alongside the body content hole), `open` controls the
 * default state. Authored via the CLI `--node`; a GUI node-view is deferred.
 */
export const Details = Node.create({
  name: "details",
  group: "block",
  content: "block+",
  defining: true,
  addAttributes() {
    return {
      summary: { default: "Details", renderHTML: () => ({}) },
      open: {
        default: false,
        parseHTML: (el: HTMLElement) => el.hasAttribute("open"),
        renderHTML: (attrs: Record<string, unknown>) =>
          attrs.open ? { open: "" } : {},
      },
    };
  },
  parseHTML() {
    return [
      {
        tag: "details",
        getAttrs: (el: HTMLElement) => ({
          summary: el.querySelector("summary")?.textContent ?? "Details",
          open: el.hasAttribute("open"),
        }),
      },
    ];
  },
  renderHTML({ node, HTMLAttributes }) {
    // The content hole (0) must be the sole child of its element — both the
    // static renderer and ProseMirror reject a bare `0` beside a static
    // <summary> sibling — so the body lives in its own wrapper div.
    return [
      "details",
      { class: "rd-details", ...HTMLAttributes },
      ["summary", {}, (node.attrs.summary as string | null) ?? "Details"],
      ["div", { class: "rd-details-body" }, 0],
    ];
  },
});

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
  Figure,
  Gloss,
  Sidenote,
  Callout,
  Details,
];
