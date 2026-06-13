<script lang="ts">
  import { css } from "styled-system/css";
  import type { ApiProjectMedia } from "$lib/bindings";
  import SectionHead from "./SectionHead.svelte";
  import MediaTile from "./MediaTile.svelte";
  import Lightbox from "./Lightbox.svelte";

  // The Gallery section: a §NN heading (continuing the prose numbering) over a
  // 2-up grid of tiles. On mobile it becomes a horizontal swipe carousel.
  interface Props {
    media: ApiProjectMedia[];
    n: number;
  }

  let { media, n }: Props = $props();

  let lightboxOpen = $state(false);
  let lightboxIndex = $state(0);

  function open(i: number) {
    lightboxIndex = i;
    lightboxOpen = true;
  }

  // 2-up grid that collapses into a horizontal swipe carousel (peeking the next
  // tile) on mobile.
  const galleryGrid = css({
    display: "grid",
    gridTemplateColumns: "1fr 1fr",
    gap: "16px",
    mt: "4px",
    "@media (max-width: 760px)": {
      gridTemplateColumns: "none",
      gridAutoFlow: "column",
      gridAutoColumns: "85%",
      gap: "12px",
      overflowX: "auto",
      scrollSnapType: "x mandatory",
      scrollbarWidth: "none",
      mx: "-16px",
      px: "16px",
      "&::-webkit-scrollbar": { display: "none" },
      "& > *": { scrollSnapAlign: "start" },
    },
  });
</script>

<div class={css({ mt: "6px" })}>
  <SectionHead {n} text="Gallery" />
  <div class={galleryGrid}>
    {#each media as m, i (m.id)}
      <MediaTile {m} onOpen={() => open(i)} />
    {/each}
  </div>
</div>

<Lightbox
  {media}
  bind:index={lightboxIndex}
  open={lightboxOpen}
  onClose={() => (lightboxOpen = false)}
/>
