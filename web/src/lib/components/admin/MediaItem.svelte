<script lang="ts">
  import { css, cx } from "styled-system/css";
  import { center, hstack } from "styled-system/patterns";
  import { decode } from "blurhash";
  import type { ProjectMedia } from "$lib/admin-types";
  import VideoThumbnail from "./VideoThumbnail.svelte";
  import IconX from "~icons/lucide/x";
  import IconPlay from "~icons/lucide/play";
  import IconFilm from "~icons/lucide/film";
  import IconImage from "~icons/lucide/image";

  interface Props {
    media: ProjectMedia;
    ondelete: () => void;
    class?: string;
  }

  let { media, ondelete, class: className }: Props = $props();

  // Get the best thumbnail URL (for images)
  const thumbUrl = $derived(
    media.variants.thumb?.url ??
      media.variants.medium?.url ??
      media.variants.full?.url ??
      media.variants.poster?.url,
  );

  // Get video URL (for videos)
  const videoUrl = $derived(media.variants.video?.url);

  // Decode blurhash to canvas on mount
  let canvasRef: HTMLCanvasElement | null = $state(null);
  let imageLoaded = $state(false);

  $effect(() => {
    if (canvasRef && media.blurhash && !imageLoaded) {
      try {
        const pixels = decode(media.blurhash, 32, 32);
        const ctx = canvasRef.getContext("2d");
        if (ctx) {
          const imageData = ctx.createImageData(32, 32);
          imageData.data.set(pixels);
          ctx.putImageData(imageData, 0, 0);
        }
      } catch {
        // Silently fail if blurhash is invalid
      }
    }
  });

  function handleImageLoad() {
    imageLoaded = true;
  }
</script>

<!-- Outer wrapper allows delete button to escape bounds -->
<div class={cx("group", css({ position: "relative" }), className)}>
  <!-- Media container with fixed height -->
  <div
    class={css({
      position: "relative",
      h: "28",
      rounded: "lg",
      borderWidth: "1px",
      borderColor: "admin.border",
      bg: "admin.bgSecondary",
      overflow: "hidden",
    })}
  >
    <!-- Blurhash placeholder -->
    {#if media.blurhash && !imageLoaded}
      <canvas
        bind:this={canvasRef}
        width="32"
        height="32"
        class={css({
          position: "absolute",
          inset: "0",
          w: "full",
          h: "full",
          objectFit: "cover",
        })}
      ></canvas>
    {/if}

    <!-- Actual thumbnail or video -->
    {#if media.mediaType === "video" && videoUrl}
      <!-- Video thumbnail - capture first frame to canvas -->
      <VideoThumbnail src={videoUrl} onload={handleImageLoad} />
    {:else if thumbUrl}
      <img
        src={thumbUrl}
        alt=""
        class={cx(
          css({
            position: "absolute",
            inset: "0",
            w: "full",
            h: "full",
            objectFit: "cover",
            transition: "opacity",
            transitionDuration: "200ms",
          }),
          imageLoaded ? css({ opacity: "1" }) : css({ opacity: "0" }),
        )}
        onload={handleImageLoad}
      />
    {:else}
      <!-- Fallback for missing thumbnail -->
      <div
        class={center({
          position: "absolute",
          inset: "0",
          color: "admin.textMuted",
        })}
      >
        {#if media.mediaType === "video"}
          <IconFilm class={css({ w: "6", h: "6" })} />
        {:else}
          <IconImage class={css({ w: "6", h: "6" })} />
        {/if}
      </div>
    {/if}

    <!-- Video badge -->
    {#if media.mediaType === "video"}
      <div
        class={hstack({
          gap: "1",
          position: "absolute",
          top: "2",
          left: "2",
          bg: "black/70",
          color: "white",
          fontSize: "xs",
          px: "1.5",
          py: "0.5",
          rounded: "sm",
        })}
      >
        <IconPlay class={css({ w: "2.5", h: "2.5" })} />
        <span>Video</span>
      </div>
    {/if}
  </div>

  <!-- Delete button - positioned outside the overflow-hidden container -->
  <button
    type="button"
    onclick={ondelete}
    class={center({
      position: "absolute",
      top: "-2",
      right: "-2",
      w: "6",
      h: "6",
      bg: "red.600",
      _hover: { bg: "red.500" },
      color: "white",
      rounded: "full",
      opacity: "0",
      _groupHover: { opacity: "1" },
      transition: "opacity",
      shadow: "md",
      zIndex: 10,
    })}
    aria-label="Delete media"
  >
    <IconX class={css({ w: "3.5", h: "3.5" })} />
  </button>
</div>
