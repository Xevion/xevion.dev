import { Extension, type Editor, type Range } from "@tiptap/core";
import { Suggestion, exitSuggestion } from "@tiptap/suggestion";
import { computePosition, flip, offset, shift } from "@floating-ui/dom";
import { mount, unmount } from "svelte";
import SlashMenu from "./SlashMenu.svelte";

/** One entry in the slash menu; `command` performs the block transformation. */
export type SlashItem = {
  title: string;
  aliases: string[];
  command: (props: { editor: Editor; range: Range }) => void;
};

/**
 * Reactive bridge between the Suggestion ProseMirror plugin (which owns the
 * query + keyboard) and the mounted Svelte popup (which renders + clicks). The
 * plugin mutates these fields; the popup reads them. A `$state` proxy in this
 * `.svelte.ts` module keeps the reads reactive across the `mount()` boundary.
 */
export type SlashMenuState = {
  items: SlashItem[];
  selectedIndex: number;
  select: (item: SlashItem) => void;
};

// Block types the menu can produce — mirrors the toolbar and the schema's block
// nodes (headings capped at 2–4 by starterKitOptions). Each command deletes the
// "/query" range first, then applies the transformation to the now-empty block.
const SLASH_ITEMS: SlashItem[] = [
  {
    title: "Text",
    aliases: ["paragraph", "p", "body"],
    command: ({ editor, range }) =>
      editor.chain().focus().deleteRange(range).setNode("paragraph").run(),
  },
  {
    title: "Heading 2",
    aliases: ["h2", "title", "large"],
    command: ({ editor, range }) =>
      editor
        .chain()
        .focus()
        .deleteRange(range)
        .setNode("heading", { level: 2 })
        .run(),
  },
  {
    title: "Heading 3",
    aliases: ["h3", "subtitle", "medium"],
    command: ({ editor, range }) =>
      editor
        .chain()
        .focus()
        .deleteRange(range)
        .setNode("heading", { level: 3 })
        .run(),
  },
  {
    title: "Heading 4",
    aliases: ["h4", "small"],
    command: ({ editor, range }) =>
      editor
        .chain()
        .focus()
        .deleteRange(range)
        .setNode("heading", { level: 4 })
        .run(),
  },
  {
    title: "Bullet list",
    aliases: ["ul", "unordered", "bullet"],
    command: ({ editor, range }) =>
      editor.chain().focus().deleteRange(range).toggleBulletList().run(),
  },
  {
    title: "Numbered list",
    aliases: ["ol", "ordered", "number"],
    command: ({ editor, range }) =>
      editor.chain().focus().deleteRange(range).toggleOrderedList().run(),
  },
  {
    title: "Quote",
    aliases: ["blockquote", "citation"],
    command: ({ editor, range }) =>
      editor.chain().focus().deleteRange(range).toggleBlockquote().run(),
  },
  {
    title: "Code block",
    aliases: ["code", "pre", "snippet"],
    command: ({ editor, range }) =>
      editor.chain().focus().deleteRange(range).toggleCodeBlock().run(),
  },
  {
    title: "Divider",
    aliases: ["hr", "rule", "separator", "---"],
    command: ({ editor, range }) =>
      editor.chain().focus().deleteRange(range).setHorizontalRule().run(),
  },
];

function filterItems(query: string): SlashItem[] {
  const q = query.trim().toLowerCase();
  if (!q) return SLASH_ITEMS;
  return SLASH_ITEMS.filter(
    (item) =>
      item.title.toLowerCase().includes(q) ||
      item.aliases.some((alias) => alias.includes(q)),
  );
}

/**
 * Notion-style "/" menu for inserting block nodes. Editor-only chrome — it adds
 * no schema, so it stays out of the shared extension arrays and is composed into
 * the editor in ContentEditor.svelte. Built on `@tiptap/suggestion`; the popup is
 * a Svelte component mounted imperatively and positioned with floating-ui against
 * the caret rect.
 */
export const SlashCommand = Extension.create({
  name: "slashCommand",

  addProseMirrorPlugins() {
    return [
      Suggestion<SlashItem, SlashItem>({
        editor: this.editor,
        char: "/",
        command: ({ editor, range, props }) => props.command({ editor, range }),
        items: ({ query }) => filterItems(query),
        render: () => {
          const menu: SlashMenuState = $state({
            items: [],
            selectedIndex: 0,
            select: () => {},
          });

          let popup: HTMLElement | null = null;
          let app: ReturnType<typeof mount> | null = null;

          const reposition = (rect: SlashCaretRect) => {
            if (!popup || !rect) return;
            const reference = {
              getBoundingClientRect: () => rect() ?? new DOMRect(),
            };
            void computePosition(reference, popup, {
              placement: "bottom-start",
              strategy: "fixed",
              middleware: [offset(6), flip(), shift({ padding: 8 })],
            }).then(({ x, y }) => {
              if (!popup) return;
              popup.style.left = `${x}px`;
              popup.style.top = `${y}px`;
            });
          };

          return {
            onStart: (props) => {
              menu.items = props.items;
              menu.selectedIndex = 0;
              menu.select = (item) => props.command(item);

              popup = document.createElement("div");
              popup.style.position = "fixed";
              popup.style.zIndex = "60";
              document.body.appendChild(popup);
              app = mount(SlashMenu, { target: popup, props: { menu } });
              reposition(props.clientRect);
            },
            onUpdate: (props) => {
              menu.items = props.items;
              menu.selectedIndex = 0;
              menu.select = (item) => props.command(item);
              reposition(props.clientRect);
            },
            onKeyDown: ({ event, view }) => {
              if (menu.items.length === 0) return false;
              if (event.key === "ArrowDown") {
                menu.selectedIndex =
                  (menu.selectedIndex + 1) % menu.items.length;
                return true;
              }
              if (event.key === "ArrowUp") {
                menu.selectedIndex =
                  (menu.selectedIndex - 1 + menu.items.length) %
                  menu.items.length;
                return true;
              }
              if (event.key === "Enter") {
                menu.select(menu.items[menu.selectedIndex]);
                return true;
              }
              if (event.key === "Escape") {
                exitSuggestion(view);
                return true;
              }
              return false;
            },
            onExit: () => {
              if (app) unmount(app);
              popup?.remove();
              app = null;
              popup = null;
            },
          };
        },
      }),
    ];
  },
});

type SlashCaretRect = (() => DOMRect | null) | null | undefined;
