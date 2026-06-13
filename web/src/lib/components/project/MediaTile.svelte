<script lang="ts">
  import { css } from "styled-system/css";
  import type { ApiProjectMedia } from "$lib/bindings";
  import GalleryVideo from "./GalleryVideo.svelte";
  import IconExpand from "~icons/lucide/maximize-2";

  // One gallery tile: an image (the whole tile opens the lightbox) or a video
  // (delegated to GalleryVideo). Caption is a bold label + gray description.
  interface Props {
    m: ApiProjectMedia;
    onOpen: () => void;
  }

  let { m, onOpen }: Props = $props();

  const isVideo = $derived(m.mediaType === "video");
  const imageUrl = $derived(
    m.variants.medium?.url ??
      m.variants.full?.url ??
      m.variants.thumb?.url ??
      null,
  );
  const posterUrl = $derived(
    m.variants.poster?.url ?? m.variants.thumb?.url ?? undefined,
  );
  const label = $derived(m.metadata?.label ?? null);
  const caption = $derived(m.metadata?.caption ?? null);

  const mediaFill = css({
    position: "absolute",
    inset: "0",
    w: "full",
    h: "full",
    objectFit: "cover",
  });
</script>

<figure class={css({ m: "0" })}>
  <div
    class={css({
      position: "relative",
      aspectRatio: "16 / 10",
      rounded: "10px",
      overflow: "hidden",
      borderWidth: "1px",
      borderColor: "zinc.200",
      bg: "surface.secondary",
      _dark: { borderColor: "zinc.700" },
    })}
  >
    {#if isVideo && m.variants.video}
      <GalleryVideo src={m.variants.video.url} poster={posterUrl} {onOpen} />
    {:else if imageUrl}
      <img
        src={imageUrl}
        alt={m.metadata?.altText ?? label ?? ""}
        loading="lazy"
        class={mediaFill}
      />
      <button
        type="button"
        onclick={onOpen}
        aria-label="Open in lightbox"
        class={css({
          position: "absolute",
          inset: "0",
          display: "grid",
          placeItems: "start end",
          p: "10px",
          bg: "transparent",
          border: "none",
          cursor: "pointer",
          transition: "background .16s ease",
          _hover: { bg: "rgba(24,24,27,.05)" },
        })}
      >
        <span
          class={css({
            display: "grid",
            placeItems: "center",
            w: "28px",
            h: "28px",
            rounded: "7px",
            bg: "overlay.badge",
            color: "zinc.600",
            borderWidth: "1px",
            borderColor: "zinc.200",
          })}
        >
          <IconExpand class={css({ w: "13px", h: "13px" })} />
        </span>
      </button>
    {/if}
  </div>

  {#if label || caption}
    <figcaption
      class={css({
        mt: "8px",
        fontSize: "caption",
        lineHeight: "1.4",
        color: "zinc.500",
        _dark: { color: "zinc.400" },
      })}
    >
      {#if label}<span
          class={css({
            color: "zinc.700",
            fontWeight: "500",
            _dark: { color: "zinc.300" },
          })}>{label}.</span
        >{/if}
      {caption}
    </figcaption>
  {/if}
</figure>
