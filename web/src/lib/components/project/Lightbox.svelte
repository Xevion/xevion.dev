<script lang="ts">
  import { css } from "styled-system/css";
  import type { ApiProjectMedia } from "$lib/bindings";
  import IconClose from "~icons/lucide/x";
  import IconPrev from "~icons/lucide/chevron-left";
  import IconNext from "~icons/lucide/chevron-right";

  // Full-screen media viewer built on a native <dialog> opened with showModal():
  // it renders in the browser top layer, so it escapes any ancestor stacking
  // context (the theme toggle no longer sits over the close button) and gets a
  // focus trap, Esc-to-close, and an inert background for free. ←/→ navigate.
  interface Props {
    media: ApiProjectMedia[];
    index: number;
    open: boolean;
    onClose: () => void;
  }

  let { media, index = $bindable(0), open, onClose }: Props = $props();

  let dialog = $state<HTMLDialogElement | null>(null);

  const current = $derived(media[index]);
  const imageUrl = $derived(
    current?.variants.full?.url ??
      current?.variants.original?.url ??
      current?.variants.medium?.url ??
      current?.variants.thumb?.url ??
      null,
  );
  const label = $derived(current?.metadata?.label ?? null);
  const caption = $derived(current?.metadata?.caption ?? null);

  function prev() {
    index = (index - 1 + media.length) % media.length;
  }
  function next() {
    index = (index + 1) % media.length;
  }
  function onKey(e: KeyboardEvent) {
    if (e.key === "ArrowLeft") prev();
    else if (e.key === "ArrowRight") next();
    // Esc is handled natively: it fires `cancel` → `close`.
  }

  // Drive the dialog from `open`. Keeping the element mounted (rather than
  // {#if}-toggling it) lets the CSS open/close transitions play out.
  $effect(() => {
    const d = dialog;
    if (!d) return;
    if (open && !d.open) d.showModal();
    else if (!open && d.open) d.close();
  });

  const navBtn = css({
    position: "absolute",
    top: "50%",
    transform: "translateY(-50%)",
    display: "grid",
    placeItems: "center",
    w: "44px",
    h: "44px",
    rounded: "full",
    bg: "overlay.control",
    color: "white",
    border: "none",
    cursor: "pointer",
    transition: "background .14s ease",
    _hover: { bg: "overlay.controlHover" },
  });
</script>

<dialog
  bind:this={dialog}
  class="rd-lightbox"
  aria-label={label ?? "Media viewer"}
  onclose={onClose}
  onkeydown={onKey}
  onclick={(e) => {
    if (e.target === dialog) onClose();
  }}
>
  {#if current}
    <button
      type="button"
      onclick={onClose}
      aria-label="Close"
      class={css({
        position: "absolute",
        top: "20px",
        right: "20px",
        display: "grid",
        placeItems: "center",
        w: "40px",
        h: "40px",
        rounded: "full",
        bg: "overlay.control",
        color: "white",
        border: "none",
        cursor: "pointer",
        _hover: { bg: "overlay.controlHover" },
      })}
    >
      <IconClose class={css({ w: "20px", h: "20px" })} />
    </button>

    {#if media.length > 1}
      <button
        type="button"
        class={navBtn}
        style="left: 20px"
        aria-label="Previous"
        onclick={prev}
      >
        <IconPrev class={css({ w: "22px", h: "22px" })} />
      </button>
      <button
        type="button"
        class={navBtn}
        style="right: 20px"
        aria-label="Next"
        onclick={next}
      >
        <IconNext class={css({ w: "22px", h: "22px" })} />
      </button>
    {/if}

    <figure
      class={css({
        m: "0",
        maxW: "min(1100px, 100%)",
        maxH: "82vh",
        display: "flex",
        flexDirection: "column",
        gap: "12px",
        alignItems: "center",
      })}
    >
      {#if current.mediaType === "video" && current.variants.video}
        <!-- svelte-ignore a11y_media_has_caption -->
        <video
          src={current.variants.video.url}
          poster={current.variants.poster?.url ?? current.variants.thumb?.url}
          controls
          autoplay
          loop
          playsinline
          class={css({
            maxW: "full",
            maxH: "78vh",
            rounded: "8px",
            display: "block",
          })}
        ></video>
      {:else if imageUrl}
        <img
          src={imageUrl}
          alt={current.metadata?.altText ?? label ?? ""}
          class={css({
            maxW: "full",
            maxH: "78vh",
            rounded: "8px",
            display: "block",
            objectFit: "contain",
          })}
        />
      {/if}
      {#if label || caption}
        <figcaption
          class={css({
            fontFamily: "schibsted",
            fontSize: "13px",
            color: "zinc.300",
            textAlign: "center",
            maxW: "640px",
          })}
        >
          {#if label}<b class={css({ color: "white", fontWeight: "600" })}
              >{label}.</b
            >{/if}
          {caption}
        </figcaption>
      {/if}
    </figure>
  {/if}
</dialog>

<style>
  /* `:global` so Svelte's CSS pruning doesn't drop the `[open]` / `::backdrop`
     state selectors (no static markup matches them). The dialog class itself is
     unique to this component, so this stays effectively scoped. */
  :global(dialog.rd-lightbox) {
    margin: auto;
    width: 100%;
    height: 100%;
    max-width: 100vw;
    max-height: 100vh;
    padding: 40px;
    border: none;
    background: transparent;
    opacity: 0;
    transform: scale(0.97);
    transition:
      opacity 0.2s ease,
      transform 0.2s ease,
      overlay 0.2s ease allow-discrete,
      display 0.2s ease allow-discrete;
  }
  :global(dialog.rd-lightbox[open]) {
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 1;
    transform: scale(1);
  }
  @starting-style {
    :global(dialog.rd-lightbox[open]) {
      opacity: 0;
      transform: scale(0.97);
    }
  }
  :global(dialog.rd-lightbox::backdrop) {
    background: rgba(9, 9, 11, 0.9);
    opacity: 0;
    transition:
      opacity 0.2s ease,
      overlay 0.2s ease allow-discrete,
      display 0.2s ease allow-discrete;
  }
  :global(dialog.rd-lightbox[open]::backdrop) {
    opacity: 1;
  }
  @starting-style {
    :global(dialog.rd-lightbox[open]::backdrop) {
      opacity: 0;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    :global(dialog.rd-lightbox),
    :global(dialog.rd-lightbox::backdrop) {
      transition: none;
    }
  }
</style>
