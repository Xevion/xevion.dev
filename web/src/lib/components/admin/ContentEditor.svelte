<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import type { Readable } from "svelte/store";
  import { createEditor, Editor, EditorContent } from "svelte-tiptap";
  import type { JSONContent } from "@tiptap/core";
  import { editorExtensions } from "$lib/tiptap/extensions.editor";
  import { codeLanguages } from "$lib/tiptap/languages";
  import { css, cx } from "styled-system/css";
  import { flex } from "styled-system/patterns";
  import { labelClass, helpTextClass } from "$lib/styles/admin";
  import { toast } from "$lib/toast";

  interface Props {
    label?: string;
    help?: string;
    content?: JSONContent | null;
  }

  let { label, help, content = $bindable(null) }: Props = $props();

  let editor = $state() as Readable<Editor>;

  onMount(() => {
    editor = createEditor({
      extensions: editorExtensions,
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

  const inCodeBlock = $derived(editor ? $editor.isActive("codeBlock") : false);
  const currentLanguage = $derived(
    editor
      ? (($editor.getAttributes("codeBlock").language as string | null) ??
          "text")
      : "text",
  );

  function setCodeLanguage(language: string) {
    $editor.chain().focus().updateAttributes("codeBlock", { language }).run();
  }

  function setLink() {
    const previous = $editor.getAttributes("link").href as string | undefined;
    const url = window.prompt("Link URL", previous ?? "");
    if (url === null) return;
    if (url === "") {
      $editor.chain().focus().extendMarkRange("link").unsetLink().run();
      return;
    }
    // Mirror the server sanitizer's allowed schemes. The public page strips a
    // bad href, but the editor renders authored links live, so reject anything
    // other than http(s)/mailto here too (no javascript:/data: in the surface).
    if (!/^(https?:|mailto:)/i.test(url)) {
      toast.error("Links must start with http://, https://, or mailto:");
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

  const languageSelectClass = css({
    ml: "1",
    px: "1.5",
    py: "0.5",
    rounded: "sm",
    fontSize: "xs",
    fontFamily: "mono",
    color: "admin.text",
    bg: "admin.bgSecondary",
    borderWidth: "1px",
    borderColor: "admin.border",
    cursor: "pointer",
    _focus: { outline: "none", borderColor: "admin.accent" },
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
            label: "{ }",
            title: "Code block",
            isActive: () => $editor.isActive("codeBlock"),
            run: () => $editor.chain().focus().toggleCodeBlock().run(),
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

        {#if inCodeBlock}
          <select
            title="Code language"
            class={languageSelectClass}
            value={currentLanguage}
            onchange={(e) => setCodeLanguage(e.currentTarget.value)}
          >
            {#each codeLanguages as lang (lang.id)}
              <option value={lang.id}>{lang.label}</option>
            {/each}
          </select>
        {/if}
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
          "& .ProseMirror": { outline: "none", minH: "14rem" },
          "& .ProseMirror p": { my: "2" },
          "& .ProseMirror h2": {
            fontSize: "xl",
            fontWeight: "bold",
            mt: "4",
            mb: "2",
          },
          "& .ProseMirror h3": {
            fontSize: "lg",
            fontWeight: "semibold",
            mt: "3",
            mb: "1.5",
          },
          "& .ProseMirror ul": { listStyle: "disc", pl: "5", my: "2" },
          "& .ProseMirror ol": {
            listStyle: "decimal",
            pl: "5",
            my: "2",
          },
          "& .ProseMirror blockquote": {
            borderLeftWidth: "3px",
            borderColor: "admin.border",
            pl: "3",
            color: "admin.textSecondary",
            my: "2",
          },
          "& .ProseMirror a": {
            color: "admin.accent",
            textDecoration: "underline",
          },
          "& .ProseMirror code": {
            bg: "admin.surfaceHover",
            px: "1",
            rounded: "sm",
            fontFamily: "mono",
            fontSize: "0.85em",
          },
          // NOTE: code-block (`pre`) + lowlight `.hljs-*` token styling lives in
          // the component style block below, not here — see the comment there.
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

<style>
  /* Code blocks are styled here with plain global CSS, not Panda css(), because
     ProseMirror's editor DOM and lowlight's hljs decoration spans are injected at
     runtime — :global() is the reliable way to reach them (Panda's `& .x`
     descendant selectors proved fragile against injected nodes).

     The palette is github light/dark, expressed as custom properties on the `pre`
     and flipped as one block under an .dark/[data-theme=dark] ancestor — so the
     in-editor preview tracks the admin theme and matches the Shiki-rendered public
     page (which uses the same github-light/github-dark themes). Token rules read
     the vars, so dark mode needs no per-token duplication. */
  :global(.tiptap-content .ProseMirror pre) {
    /* --code-canvas / --code-scrollbar are shared with the public page via
       globalCss (panda.config.ts); the --cb-* token vars are editor-only. */
    --cb-fg: #24292e;
    --cb-border: #d0d7de;
    --cb-comment: #6a737d;
    --cb-keyword: #d73a49;
    --cb-title: #6f42c1;
    --cb-attr: #005cc5;
    --cb-string: #032f62;
    --cb-builtin: #e36209;
    --cb-name: #22863a;

    background-color: var(--code-canvas);
    color: var(--cb-fg);
    font-family: var(--fonts-mono, ui-monospace, monospace);
    font-size: 0.8125rem;
    line-height: 1.6;
    padding: 0.75rem 0.875rem;
    margin: 0.75rem 0;
    border: 1px solid var(--cb-border);
    border-radius: 0.375rem;
    max-height: 24rem;
    overflow: auto;
    tab-size: 2;
    -moz-tab-size: 2;
    /* Firefox: thin themed scrollbar over the (transparent) code surface. */
    scrollbar-width: thin;
    scrollbar-color: var(--code-scrollbar) transparent;
  }
  :global(.dark .tiptap-content .ProseMirror pre),
  :global([data-theme="dark"] .tiptap-content .ProseMirror pre) {
    --cb-fg: #c9d1d9;
    --cb-border: #30363d;
    --cb-comment: #8b949e;
    --cb-keyword: #ff7b72;
    --cb-title: #d2a8ff;
    --cb-attr: #79c0ff;
    --cb-string: #a5d6ff;
    --cb-builtin: #ffa657;
    --cb-name: #7ee787;
  }
  /* WebKit/Blink: the OS overlay scrollbar is near-invisible on the code surface,
     so give it an explicit thin themed thumb. */
  :global(.tiptap-content .ProseMirror pre::-webkit-scrollbar) {
    width: 0.5rem;
    height: 0.5rem;
  }
  :global(.tiptap-content .ProseMirror pre::-webkit-scrollbar-track) {
    background: transparent;
  }
  :global(.tiptap-content .ProseMirror pre::-webkit-scrollbar-thumb) {
    background-color: var(--code-scrollbar);
    border-radius: 0.25rem;
  }
  :global(.tiptap-content .ProseMirror pre code) {
    background: none;
    color: inherit;
    padding: 0;
    font-size: inherit;
    border-radius: 0;
  }
  /* The themed surface + permanent border delineate the block; replace ProseMirror's
     node-selection outline (which overlapped the caret) with an offset accent ring. */
  :global(.tiptap-content .ProseMirror pre.ProseMirror-selectednode) {
    outline: 2px solid var(--colors-admin-accent, #6366f1);
    outline-offset: 2px;
  }

  /* lowlight (highlight.js) token scopes → github palette, mapped through the vars. */
  :global(.tiptap-content .ProseMirror pre .hljs-comment),
  :global(.tiptap-content .ProseMirror pre .hljs-code),
  :global(.tiptap-content .ProseMirror pre .hljs-formula) {
    color: var(--cb-comment);
    font-style: italic;
  }
  :global(.tiptap-content .ProseMirror pre .hljs-keyword),
  :global(.tiptap-content .ProseMirror pre .hljs-doctag),
  :global(.tiptap-content .ProseMirror pre .hljs-type),
  :global(.tiptap-content .ProseMirror pre .hljs-template-tag),
  :global(.tiptap-content .ProseMirror pre .hljs-template-variable),
  :global(.tiptap-content .ProseMirror pre .hljs-variable.language_) {
    color: var(--cb-keyword);
  }
  :global(.tiptap-content .ProseMirror pre .hljs-title),
  :global(.tiptap-content .ProseMirror pre .hljs-title.class_),
  :global(.tiptap-content .ProseMirror pre .hljs-title.function_) {
    color: var(--cb-title);
  }
  :global(.tiptap-content .ProseMirror pre .hljs-attr),
  :global(.tiptap-content .ProseMirror pre .hljs-attribute),
  :global(.tiptap-content .ProseMirror pre .hljs-literal),
  :global(.tiptap-content .ProseMirror pre .hljs-meta),
  :global(.tiptap-content .ProseMirror pre .hljs-number),
  :global(.tiptap-content .ProseMirror pre .hljs-operator),
  :global(.tiptap-content .ProseMirror pre .hljs-variable),
  :global(.tiptap-content .ProseMirror pre .hljs-selector-attr),
  :global(.tiptap-content .ProseMirror pre .hljs-selector-class),
  :global(.tiptap-content .ProseMirror pre .hljs-selector-id) {
    color: var(--cb-attr);
  }
  :global(.tiptap-content .ProseMirror pre .hljs-string),
  :global(.tiptap-content .ProseMirror pre .hljs-regexp),
  :global(.tiptap-content .ProseMirror pre .hljs-meta .hljs-string) {
    color: var(--cb-string);
  }
  :global(.tiptap-content .ProseMirror pre .hljs-built_in),
  :global(.tiptap-content .ProseMirror pre .hljs-symbol) {
    color: var(--cb-builtin);
  }
  :global(.tiptap-content .ProseMirror pre .hljs-name),
  :global(.tiptap-content .ProseMirror pre .hljs-quote),
  :global(.tiptap-content .ProseMirror pre .hljs-selector-tag),
  :global(.tiptap-content .ProseMirror pre .hljs-selector-pseudo) {
    color: var(--cb-name);
  }
</style>
