<script lang="ts">
  import TagChip from "./TagChip.svelte";
  import OverflowPill from "./OverflowPill.svelte";

  export type Tag = { iconSvg?: string; name: string; color?: string };

  interface Props {
    tags: Tag[];
    maxRows?: number;
    class?: string;
    style?: string;
  }

  let { tags, maxRows = 2, class: className, style }: Props = $props();

  // Tag overflow detection
  let tagsContainer: HTMLDivElement | null = $state(null);
  let visibleTagCount: number | null = $state(null); // null = show all

  // Derived visible and hidden tags based on overflow calculation
  const effectiveVisibleCount = $derived(visibleTagCount ?? tags.length);
  const visibleTags = $derived(tags.slice(0, effectiveVisibleCount));
  const hiddenTags = $derived(tags.slice(effectiveVisibleCount));
  const hiddenTagNames = $derived(hiddenTags.map((t) => t.name));

  // Measure and calculate tag overflow
  function calculateOverflow() {
    if (!tagsContainer || tags.length === 0) return;

    const container = tagsContainer;
    const children = Array.from(container.children) as HTMLElement[];
    if (children.length === 0) return;

    // Get computed gap from container
    const containerStyle = getComputedStyle(container);
    const gap = parseFloat(containerStyle.gap) || 4;

    // Measure first tag to get line height
    const firstTag = children[0];
    if (!firstTag) return;
    const lineHeight = firstTag.offsetHeight;
    const maxHeight = lineHeight * maxRows + gap * (maxRows - 1);

    // If container fits within max height, show all tags
    if (container.scrollHeight <= maxHeight + 1) {
      visibleTagCount = null; // Show all
      return;
    }

    // Binary search to find optimal visible count
    let low = 1;
    let high = tags.length;
    let result = 1;

    while (low <= high) {
      const mid = Math.floor((low + high) / 2);

      // Temporarily show only 'mid' tags to measure
      visibleTagCount = mid;

      // Force reflow to get accurate measurement
      // eslint-disable-next-line @typescript-eslint/no-unused-expressions
      container.offsetHeight;

      // Measure based on children positions
      const currentHiddenCount = tags.length - mid;
      const visibleChildren = Array.from(container.children).slice(
        0,
        mid + (currentHiddenCount > 0 ? 1 : 0),
      ) as HTMLElement[];

      if (visibleChildren.length === 0) {
        low = mid + 1;
        continue;
      }

      // Calculate actual height based on child positions
      let minTop = Infinity;
      let maxBottom = 0;
      for (const child of visibleChildren) {
        const rect = child.getBoundingClientRect();
        const containerRect = container.getBoundingClientRect();
        const relativeTop = rect.top - containerRect.top;
        const relativeBottom = rect.bottom - containerRect.top;
        minTop = Math.min(minTop, relativeTop);
        maxBottom = Math.max(maxBottom, relativeBottom);
      }
      const actualHeight = maxBottom - minTop;

      if (actualHeight <= maxHeight + 1) {
        result = mid;
        low = mid + 1;
      } else {
        high = mid - 1;
      }
    }

    // Set final visible count
    if (result < tags.length) {
      visibleTagCount = result;
    } else {
      visibleTagCount = null; // Show all
    }
  }

  // Run overflow calculation after mount and on resize
  $effect(() => {
    if (!tagsContainer) return;

    // Initial calculation
    calculateOverflow();

    // Set up resize observer
    const resizeObserver = new ResizeObserver(() => {
      // Reset to show all tags first, then recalculate
      visibleTagCount = null;
      // Use requestAnimationFrame to ensure DOM has updated
      requestAnimationFrame(() => {
        calculateOverflow();
      });
    });

    resizeObserver.observe(tagsContainer);

    return () => {
      resizeObserver.disconnect();
    };
  });
</script>

<div
  bind:this={tagsContainer}
  class="flex flex-row-reverse flex-wrap-reverse gap-1 {className}"
  {style}
>
  {#each visibleTags as tag (tag.name)}
    <TagChip name={tag.name} color={tag.color} iconSvg={tag.iconSvg} />
  {/each}
  {#if hiddenTags.length > 0}
    <OverflowPill count={hiddenTags.length} {hiddenTagNames} />
  {/if}
</div>
