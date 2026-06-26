<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import type { Readable } from "svelte/store";
  import {
    createEditor,
    Editor,
    EditorContent,
    BubbleMenu,
  } from "svelte-tiptap";
  import type { JSONContent } from "@tiptap/core";
  import { DragHandle } from "@tiptap/extension-drag-handle";
  import { offset } from "@floating-ui/dom";
  import { editorExtensions } from "$lib/tiptap/extensions.editor";
  import { SlashCommand } from "$lib/tiptap/slash-command.svelte";
  import { codeLanguages } from "$lib/tiptap/languages";
  import { codeTokens } from "$lib/tiptap/code-tokens";
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

  // grip-vertical (lucide). Inlined as raw SVG because DragHandle.render builds a
  // plain DOM element, where the project's ~icons Svelte component can't be used.
  const GRIP_SVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="9" cy="12" r="1"/><circle cx="9" cy="5" r="1"/><circle cx="9" cy="19" r="1"/><circle cx="15" cy="12" r="1"/><circle cx="15" cy="5" r="1"/><circle cx="15" cy="19" r="1"/></svg>`;

  const dragHandle = DragHandle.configure({
    // Target top-level blocks only: a whole list/blockquote drags as one unit,
    // never an individual list item. Matches the document's block model (the
    // doc's direct children are "the blocks") and is easier to aim than nested.
    nested: false,
    render: () => {
      const el = document.createElement("div");
      el.className = "tiptap-drag-handle";
      el.setAttribute("role", "button");
      el.setAttribute("aria-label", "Drag to reorder block");
      el.innerHTML = GRIP_SVG;
      return el;
    },
    // `fixed` escapes the editor wrapper's `overflow: hidden` (the handle is
    // appended inside it); the offset keeps the grip in the left gutter rather
    // than flush against the block's edge.
    computePositionConfig: {
      placement: "left-start",
      strategy: "fixed",
      middleware: [offset(4)],
    },
  });

  onMount(() => {
    editor = createEditor({
      // Block chrome (slash menu, drag handle) is editor-only behavior with no
      // schema impact, so it's composed here rather than in the shared arrays.
      extensions: [...editorExtensions, SlashCommand, dragHandle],
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

  // Bubble-menu state: which inline mark is under the cursor and its attrs.
  const linkActive = $derived(editor ? $editor.isActive("link") : false);
  const codeActive = $derived(editor ? $editor.isActive("code") : false);
  const currentHref = $derived(
    editor
      ? (($editor.getAttributes("link").href as string | undefined) ?? "")
      : "",
  );
  const inlineLang = $derived(
    editor
      ? (($editor.getAttributes("code").lang as string | null) ?? "text")
      : "text",
  );
  const inlineToken = $derived(
    editor
      ? (($editor.getAttributes("code").token as string | null) ?? "")
      : "",
  );

  function setCodeLanguage(language: string) {
    $editor.chain().focus().updateAttributes("codeBlock", { language }).run();
  }

  // Inline-mark editing (link href, inline-code language/token) lives in the
  // bubble menu. The toolbar "Link" button only seeds a link the bubble then
  // edits — replacing the old window.prompt, whose empty-prefill branch deleted
  // the link on a code+link span (the prompt came back empty there).
  // Requires the authority slashes (`http://`, not `http:`) so it matches the
  // server validator (pm.rs `LINK_SCHEMES`) exactly — an editor that accepted
  // `http:foo` would pass a link the write path then rejects.
  const LINK_SCHEME = /^(https?:\/\/|mailto:)/i;
  // Seeded over a fresh selection; the bubble's input completes it. Treated as
  // empty on commit so an untouched seed removes itself instead of shipping a
  // dead bare-scheme link.
  const LINK_SEED = "https://";

  function startLink() {
    // An active link already shows the bubble (shouldShow), so there's nothing to
    // seed — the user edits it there.
    if ($editor.isActive("link")) return;
    if ($editor.state.selection.empty) {
      toast.error("Select text to turn into a link");
      return;
    }
    $editor.chain().focus().setLink({ href: LINK_SEED }).run();
  }

  function applyLink(href: string) {
    const trimmed = href.trim();
    // A cleared field or an untouched seed means "no link".
    if (trimmed === "" || trimmed === LINK_SEED) {
      removeLink();
      return;
    }
    // Unchanged — skip the transaction so a stray blur doesn't churn undo history.
    if (trimmed === currentHref) return;
    // Mirror the server sanitizer's allowed schemes — the editor renders authored
    // links live, so reject anything but http(s)/mailto here too.
    if (!LINK_SCHEME.test(trimmed)) {
      toast.error("Links must start with http://, https://, or mailto:");
      return;
    }
    $editor
      .chain()
      .focus()
      .extendMarkRange("link")
      .setLink({ href: trimmed })
      .run();
  }

  function removeLink() {
    $editor.chain().focus().extendMarkRange("link").unsetLink().run();
  }

  // A code span is either grammar-highlighted (`lang`) or painted as a single
  // token kind (`token`), so setting one clears the other; "text"/"" clears both
  // back to a plain inline-code span.
  function setInlineLang(lang: string) {
    const attrs =
      lang === "text" ? { lang: null, token: null } : { lang, token: null };
    $editor
      .chain()
      .focus()
      .extendMarkRange("code")
      .updateAttributes("code", attrs)
      .run();
  }

  function setInlineToken(token: string) {
    const attrs = token === "" ? { token: null } : { token, lang: null };
    $editor
      .chain()
      .focus()
      .extendMarkRange("code")
      .updateAttributes("code", attrs)
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

  const bubbleClass = css({
    display: "flex",
    flexDirection: "column",
    gap: "1",
    p: "1.5",
    rounded: "md",
    borderWidth: "1px",
    borderColor: "admin.border",
    bg: "admin.surface",
    boxShadow: "0 4px 16px rgba(0, 0, 0, 0.18)",
    zIndex: 50,
  });

  const bubbleRowClass = flex({ align: "center", gap: "1" });

  const bubbleInputClass = css({
    px: "1.5",
    py: "0.5",
    w: "13rem",
    rounded: "sm",
    fontSize: "xs",
    fontFamily: "mono",
    color: "admin.text",
    bg: "admin.bgSecondary",
    borderWidth: "1px",
    borderColor: "admin.border",
    _focus: { outline: "none", borderColor: "admin.accent" },
  });

  const bubbleIconButtonClass = css({
    px: "1.5",
    py: "0.5",
    rounded: "sm",
    fontSize: "sm",
    lineHeight: "1",
    color: "admin.textSecondary",
    cursor: "pointer",
    _hover: { bg: "admin.surfaceHover", color: "admin.text" },
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
            run: startLink,
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
          pr: "3",
          py: "2",
          minH: "16rem",
          fontSize: "sm",
          color: "admin.text",
          cursor: "text",
          // The drag-handle gutter lives on .ProseMirror, not this wrapper, so the
          // gutter band is part of the editor's hover region. The extension hides
          // the handle on .ProseMirror's mouseleave unless the pointer lands
          // straight on the handle; if the gutter belonged to this wrapper, the
          // cursor would exit .ProseMirror crossing it and the handle would vanish
          // mid-reach (handle x is unchanged — it sits inside this left padding).
          "& .ProseMirror": { outline: "none", minH: "14rem", pl: "7" },
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
        <BubbleMenu
          editor={$editor}
          class={bubbleClass}
          options={{ placement: "top" }}
          shouldShow={(props) =>
            props.editor.isActive("link") || props.editor.isActive("code")}
        >
          {#if linkActive}
            <div class={bubbleRowClass}>
              <input
                type="text"
                class={bubbleInputClass}
                placeholder="https://…"
                value={currentHref}
                onkeydown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    applyLink(e.currentTarget.value);
                  }
                }}
                onblur={(e) => applyLink(e.currentTarget.value)}
              />
              <button
                type="button"
                title="Remove link"
                class={bubbleIconButtonClass}
                onclick={removeLink}
              >
                ×
              </button>
            </div>
          {/if}
          {#if codeActive}
            <div class={bubbleRowClass}>
              <select
                title="Inline code language"
                class={languageSelectClass}
                value={inlineLang}
                onchange={(e) => setInlineLang(e.currentTarget.value)}
              >
                {#each codeLanguages as lang (lang.id)}
                  <option value={lang.id}>{lang.label}</option>
                {/each}
              </select>
              <select
                title="Inline code token"
                class={languageSelectClass}
                value={inlineToken}
                onchange={(e) => setInlineToken(e.currentTarget.value)}
              >
                <option value="">— token —</option>
                {#each codeTokens as token (token.id)}
                  <option value={token.id}>{token.label}</option>
                {/each}
              </select>
            </div>
          {/if}
        </BubbleMenu>
      {/if}
    </div>
  </div>

  {#if help}
    <p class={helpTextClass}>{help}</p>
  {/if}
</div>

<style>
  /* The drag handle is raw DOM injected by the extension (not a Svelte node), so
     it's reached with :global() and themed via the admin token CSS vars. The
     extension toggles visibility on block hover; we style appearance only. */
  :global(.tiptap-drag-handle) {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 1.25rem;
    height: 1.5rem;
    border-radius: 0.25rem;
    color: var(--colors-admin-text-secondary, #71717a);
    cursor: grab;
    transition:
      background-color 0.12s,
      color 0.12s;
  }
  :global(.tiptap-drag-handle:hover) {
    background-color: var(--colors-admin-surface-hover, rgba(0, 0, 0, 0.06));
    color: var(--colors-admin-text, #18181b);
  }
  :global(.tiptap-drag-handle:active),
  :global(.tiptap-drag-handle[data-dragging="true"]) {
    cursor: grabbing;
  }
  :global(.tiptap-drag-handle svg) {
    width: 1rem;
    height: 1rem;
  }

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
