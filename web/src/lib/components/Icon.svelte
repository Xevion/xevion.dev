<script lang="ts">
  interface Props {
    icon: string; // "collection:name" format, e.g., "lucide:home"
    class?: string;
    size?: string; // Tailwind size class, e.g., "size-4"
  }

  let { icon, class: className = "", size = "size-4" }: Props = $props();

  // Validate and parse icon identifier into collection and name
  const iconUrl = $derived.by(() => {
    const colonIndex = icon.indexOf(":");
    if (
      colonIndex === -1 ||
      colonIndex === 0 ||
      colonIndex === icon.length - 1
    ) {
      console.warn(
        `Invalid icon identifier: "${icon}" (expected "collection:name" format)`,
      );
      return null;
    }
    const collection = icon.slice(0, colonIndex);
    const name = icon.slice(colonIndex + 1);
    return `/api/icons/${collection}/${name}.svg`;
  });
</script>

<!--
  CSS mask-image approach for SVG icons:
  - Browser loads SVG natively (no JS, starts at HTML parse time)
  - mask-image uses SVG shape, background-color provides the fill
  - currentColor inheritance works via background-color
  - HTTP caching handled by browser automatically
-->
{#if iconUrl}
  <span
    class="inline-block {size} {className}"
    style="
      background-color: currentColor;
      mask-image: url('{iconUrl}');
      mask-size: contain;
      mask-repeat: no-repeat;
      mask-position: center;
      -webkit-mask-image: url('{iconUrl}');
      -webkit-mask-size: contain;
      -webkit-mask-repeat: no-repeat;
      -webkit-mask-position: center;
    "
    role="img"
    aria-hidden="true"
  ></span>
{/if}
