<script lang="ts">
  import { css, cx } from "styled-system/css";

  interface Props {
    icon: string; // "collection:name" format, e.g., "lucide:home"
    class?: string;
    size?: string; // CSS size values, e.g., "4" for w/h: "4"
    sizeClass?: string; // Pre-built css() class for responsive sizes
  }

  let { icon, class: className = "", size = "4", sizeClass }: Props = $props();

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
    class={cx(
      sizeClass ?? css({ w: size, h: size }),
      css({ display: "inline-block" }),
      className,
    )}
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
