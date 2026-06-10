<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import type { Readable } from "svelte/store";
  import { createEditor, Editor, EditorContent } from "svelte-tiptap";
  import type { JSONContent } from "@tiptap/core";
  import { tiptapExtensions } from "$lib/tiptap/extensions";
  import { css, cx } from "styled-system/css";
  import { flex } from "styled-system/patterns";
  import { labelClass, helpTextClass } from "$lib/styles/admin";

  interface Props {
    label?: string;
    help?: string;
    content?: JSONContent | null;
  }

  let { label, help, content = $bindable(null) }: Props = $props();

  let editor = $state() as Readable<Editor>;

  onMount(() => {
    editor = createEditor({
      extensions: tiptapExtensions,
      content: content ?? "",
      onUpdate: ({ editor }) => {
        // Empty doc → null so the project simply has no detail page.
        content = editor.isEmpty ? null : editor.getJSON();
      },
    });
  });

  onDestroy(() => {
    if (editor) $editor.destroy();
  });

  function setLink() {
    const previous = $editor.getAttributes("link").href as string | undefined;
    const url = window.prompt("Link URL", previous ?? "");
    if (url === null) return;
    if (url === "") {
      $editor.chain().focus().extendMarkRange("link").unsetLink().run();
      return;
    }
    $editor
      .chain()
      .focus()
      .extendMarkRange("link")
      .setLink({ href: url })
      .run();
  }

  const buttonClass = css({
    px: "2",
    py: "1",
    rounded: "sm",
    fontSize: "xs",
    fontWeight: "medium",
    color: "admin.textSecondary",
    cursor: "pointer",
    transition: "colors",
    _hover: { bg: "admin.surfaceHover", color: "admin.text" },
  });

  const activeClass = css({
    bg: "admin.accent/15",
    color: "admin.accent",
  });

  const toolbarButtons: Array<{
    label: string;
    title: string;
    isActive?: () => boolean;
    run: () => void;
  }> = $derived(
    editor
      ? [
          {
            label: "B",
            title: "Bold",
            isActive: () => $editor.isActive("bold"),
            run: () => $editor.chain().focus().toggleBold().run(),
          },
          {
            label: "I",
            title: "Italic",
            isActive: () => $editor.isActive("italic"),
            run: () => $editor.chain().focus().toggleItalic().run(),
          },
          {
            label: "S",
            title: "Strikethrough",
            isActive: () => $editor.isActive("strike"),
            run: () => $editor.chain().focus().toggleStrike().run(),
          },
          {
            label: "</>",
            title: "Inline code",
            isActive: () => $editor.isActive("code"),
            run: () => $editor.chain().focus().toggleCode().run(),
          },
          {
            label: "H2",
            title: "Heading 2",
            isActive: () => $editor.isActive("heading", { level: 2 }),
            run: () =>
              $editor.chain().focus().toggleHeading({ level: 2 }).run(),
          },
          {
            label: "H3",
            title: "Heading 3",
            isActive: () => $editor.isActive("heading", { level: 3 }),
            run: () =>
              $editor.chain().focus().toggleHeading({ level: 3 }).run(),
          },
          {
            label: "• List",
            title: "Bullet list",
            isActive: () => $editor.isActive("bulletList"),
            run: () => $editor.chain().focus().toggleBulletList().run(),
          },
          {
            label: "1. List",
            title: "Ordered list",
            isActive: () => $editor.isActive("orderedList"),
            run: () => $editor.chain().focus().toggleOrderedList().run(),
          },
          {
            label: "❝",
            title: "Blockquote",
            isActive: () => $editor.isActive("blockquote"),
            run: () => $editor.chain().focus().toggleBlockquote().run(),
          },
          {
            label: "Link",
            title: "Set link",
            isActive: () => $editor.isActive("link"),
            run: setLink,
          },
          {
            label: "―",
            title: "Horizontal rule",
            run: () => $editor.chain().focus().setHorizontalRule().run(),
          },
          {
            label: "↶",
            title: "Undo",
            run: () => $editor.chain().focus().undo().run(),
          },
          {
            label: "↷",
            title: "Redo",
            run: () => $editor.chain().focus().redo().run(),
          },
        ]
      : [],
  );
</script>

<div class={css({ spaceY: "1.5" })}>
  {#if label}
    <span class={labelClass}>{label}</span>
  {/if}

  <div
    class={css({
      rounded: "md",
      borderWidth: "1px",
      borderColor: "admin.border",
      bg: "admin.bgSecondary",
      overflow: "hidden",
      _focusWithin: { borderColor: "admin.accent" },
    })}
  >
    {#if editor}
      <div
        class={cx(
          flex({ wrap: "wrap", gap: "0.5", align: "center" }),
          css({
            p: "1.5",
            borderBottomWidth: "1px",
            borderColor: "admin.border",
            bg: "admin.surface",
          }),
        )}
      >
        {#each toolbarButtons as btn (btn.title)}
          <button
            type="button"
            title={btn.title}
            class={cx(buttonClass, btn.isActive?.() ? activeClass : "")}
            onclick={btn.run}
          >
            {btn.label}
          </button>
        {/each}
      </div>
    {/if}

    <div
      class={cx(
        "tiptap-content",
        css({
          px: "3",
          py: "2",
          minH: "16rem",
          fontSize: "sm",
          color: "admin.text",
          cursor: "text",
          "& :global(.ProseMirror)": { outline: "none", minH: "14rem" },
          "& :global(.ProseMirror p)": { my: "2" },
          "& :global(.ProseMirror h2)": {
            fontSize: "xl",
            fontWeight: "bold",
            mt: "4",
            mb: "2",
          },
          "& :global(.ProseMirror h3)": {
            fontSize: "lg",
            fontWeight: "semibold",
            mt: "3",
            mb: "1.5",
          },
          "& :global(.ProseMirror ul)": { listStyle: "disc", pl: "5", my: "2" },
          "& :global(.ProseMirror ol)": {
            listStyle: "decimal",
            pl: "5",
            my: "2",
          },
          "& :global(.ProseMirror blockquote)": {
            borderLeftWidth: "3px",
            borderColor: "admin.border",
            pl: "3",
            color: "admin.textSecondary",
            my: "2",
          },
          "& :global(.ProseMirror a)": {
            color: "admin.accent",
            textDecoration: "underline",
          },
          "& :global(.ProseMirror code)": {
            bg: "admin.surfaceHover",
            px: "1",
            rounded: "sm",
            fontFamily: "mono",
            fontSize: "0.85em",
          },
        }),
      )}
    >
      {#if editor}
        <EditorContent editor={$editor} />
      {/if}
    </div>
  </div>

  {#if help}
    <p class={helpTextClass}>{help}</p>
  {/if}
</div>
