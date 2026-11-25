<script lang="ts">
  import { css, cx } from "styled-system/css";
  import { onMount } from "svelte";
  import IconExpand from "~icons/lucide/maximize-2";
  import IconVolumeOff from "~icons/lucide/volume-x";
  import IconVolumeOn from "~icons/lucide/volume-2";

  // A gallery video: autoplays muted + looped only while on-screen, with an
  // unmute toggle, a buffering spinner, and a corner expand-to-lightbox button.
  interface Props {
    src: string;
    poster?: string;
    onOpen: () => void;
  }

  let { src, poster, onOpen }: Props = $props();

  let video = $state<HTMLVideoElement | null>(null);
  let muted = $state(true);
  let buffering = $state(false);

  function toggleMute() {
    if (!video) return;
    video.muted = !video.muted;
    muted = video.muted;
  }

  onMount(() => {
    if (!video) return;
    const el = video;
    const onWaiting = () => (buffering = true);
    const onPlaying = () => (buffering = false);
    el.addEventListener("waiting", onWaiting);
    el.addEventListener("stalled", onWaiting);
    el.addEventListener("playing", onPlaying);
    el.addEventListener("canplay", onPlaying);

    const io = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (e.isIntersecting) void el.play().catch(() => {});
          else el.pause();
        }
      },
      { threshold: 0.25 },
    );
    io.observe(el);

    return () => {
      el.removeEventListener("waiting", onWaiting);
      el.removeEventListener("stalled", onWaiting);
      el.removeEventListener("playing", onPlaying);
      el.removeEventListener("canplay", onPlaying);
      io.disconnect();
    };
  });

  const cornerBtn = css({
    position: "absolute",
    display: "grid",
    placeItems: "center",
    w: "28px",
    h: "28px",
    cursor: "pointer",
    border: "none",
  });
</script>

<video
  bind:this={video}
  {src}
  {poster}
  muted
  loop
  playsinline
  preload="metadata"
  class={css({
    position: "absolute",
    inset: "0",
    w: "full",
    h: "full",
    objectFit: "cover",
  })}
></video>

<button
  type="button"
  onclick={onOpen}
  aria-label="Open in lightbox"
  class={cx(
    cornerBtn,
    css({
      top: "10px",
      right: "10px",
      rounded: "7px",
      bg: "overlay.badge",
      color: "zinc.600",
      borderWidth: "1px",
      borderColor: "zinc.200",
    }),
  )}
>
  <IconExpand class={css({ w: "13px", h: "13px" })} />
</button>

{#if buffering}
  <span
    class={css({
      position: "absolute",
      left: "50%",
      top: "50%",
      transform: "translate(-50%,-50%)",
    })}
  >
    <span class="rd-spinner"></span>
  </span>
{:else}
  <button
    type="button"
    onclick={toggleMute}
    aria-label={muted ? "Unmute" : "Mute"}
    class={cx(
      cornerBtn,
      css({
        bottom: "10px",
        left: "10px",
        rounded: "full",
        bg: "rgba(24,24,27,.72)",
        color: "white",
      }),
    )}
  >
    {#if muted}
      <IconVolumeOff class={css({ w: "13px", h: "13px" })} />
    {:else}
      <IconVolumeOn class={css({ w: "13px", h: "13px" })} />
    {/if}
  </button>
{/if}

<style>
  .rd-spinner {
    display: inline-block;
    width: 26px;
    height: 26px;
    border-radius: 999px;
    border: 2px solid rgba(82, 82, 91, 0.25);
    border-top-color: #52525b;
    animation: rd-spin 0.7s linear infinite;
  }
  @keyframes rd-spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
