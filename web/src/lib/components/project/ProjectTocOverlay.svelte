<script lang="ts">
  import { css } from "styled-system/css";
  import IconList from "~icons/lucide/list";
  import type { TocItem } from "$lib/tiptap/render.server";

  // Mobile-only table of contents: a floating button opens a bottom sheet built
  // on a native <dialog> (showModal), matching Lightbox — top layer, focus trap,
  // Esc-to-close, and an inert background for free. Desktop uses the sticky
  // in-rail ProjectToc instead, so the trigger is hidden above 760px.
  let { toc }: { toc: TocItem[] } = $props();

  let open = $state(false);
  let dialog = $state<HTMLDialogElement | null>(null);

  // Drive the dialog from `open`; keeping it mounted lets the open/close
  // transitions play (see the :global rules below).
  $effect(() => {
    const d = dialog;
    if (!d) return;
    if (open && !d.open) d.showModal();
    else if (!open && d.open) d.close();
  });

  function jump(id: string) {
    open = false;
    // scroll-margin-top on the heading keeps it clear of the viewport top.
    document.getElementById(id)?.scrollIntoView({ behavior: "smooth" });
  }

  const fab = css({
    display: "none",
    "@media (max-width: 760px)": {
      position: "fixed",
      right: "18px",
      bottom: "18px",
      zIndex: "50",
      display: "inline-flex",
      alignItems: "center",
      gap: "8px",
      px: "16px",
      py: "10px",
      rounded: "full",
      bg: "var(--accent)",
      color: "var(--accent-ink)",
      fontFamily: "geist",
      fontSize: "bodySm",
      fontWeight: "600",
      shadow: "lg",
      cursor: "pointer",
    },
  });

  const jumpItem = css({
    display: "block",
    w: "full",
    textAlign: "left",
    py: "8px",
    fontFamily: "schibsted",
    fontSize: "15px",
    lineHeight: "1.35",
    color: "text.secondary",
    bg: "transparent",
    border: "none",
    cursor: "pointer",
    _hover: { color: "var(--accent)" },
  });
</script>

<button
  class={fab}
  onclick={() => (open = true)}
  aria-label="Table of contents"
>
  <IconList class={css({ w: "16px", h: "16px" })} />
  Contents
</button>

<dialog
  bind:this={dialog}
  class="rd-toc-sheet"
  aria-label="On this page"
  onclose={() => (open = false)}
  onclick={(e) => {
    if (e.target === dialog) open = false;
  }}
>
  <span class={css({ textStyle: "label.micro" })}>On this page</span>
  <ul
    class={css({
      mt: "12px",
      display: "flex",
      flexDirection: "column",
      gap: "2px",
      listStyle: "none",
      p: "0",
      m: "0",
    })}
  >
    {#each toc as item (item.id)}
      <li>
        <button
          class={jumpItem}
          style="padding-left: {item.level === 2 ? 0 : 16}px"
          onclick={() => jump(item.id)}
        >
          {item.text}
        </button>
      </li>
    {/each}
  </ul>
</dialog>

<style>
  /* `:global` so Svelte's CSS pruning keeps the `[open]` / `::backdrop` state
     selectors (no static markup matches them); the class is unique to this
     component, so it stays effectively scoped. Mirrors Lightbox. */
  :global(dialog.rd-toc-sheet) {
    position: fixed;
    inset: auto 0 0 0;
    width: 100%;
    max-width: 100%;
    max-height: 70vh;
    margin: 0;
    padding: 20px 22px calc(20px + env(safe-area-inset-bottom));
    border: none;
    border-radius: 16px 16px 0 0;
    background: var(--colors-surface);
    color: inherit;
    overflow-y: auto;
    transform: translateY(100%);
    transition:
      transform 0.22s ease,
      overlay 0.22s ease allow-discrete,
      display 0.22s ease allow-discrete;
  }
  :global(dialog.rd-toc-sheet[open]) {
    transform: translateY(0);
  }
  @starting-style {
    :global(dialog.rd-toc-sheet[open]) {
      transform: translateY(100%);
    }
  }
  :global(dialog.rd-toc-sheet::backdrop) {
    background: rgba(9, 9, 11, 0.55);
    opacity: 0;
    transition:
      opacity 0.22s ease,
      overlay 0.22s ease allow-discrete,
      display 0.22s ease allow-discrete;
  }
  :global(dialog.rd-toc-sheet[open]::backdrop) {
    opacity: 1;
  }
  @starting-style {
    :global(dialog.rd-toc-sheet[open]::backdrop) {
      opacity: 0;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    :global(dialog.rd-toc-sheet),
    :global(dialog.rd-toc-sheet::backdrop) {
      transition: none;
    }
  }
</style>
