<script lang="ts">
  import { css, cx } from "styled-system/css";
  import { iconSvgPath, parseIconIdentifier } from "#lib/types/icons.ts";

  interface Props {
    icon: string; // "collection:name" format, e.g., "lucide:home"
    class?: string;
    size?: string; // CSS size values, e.g., "4" for w/h: "4"
    sizeClass?: string; // Pre-built css() class for responsive sizes
  }

  let { icon, class: className = "", size = "4", sizeClass }: Props = $props();

  const iconUrl = $derived.by(() => {
    const parsed = parseIconIdentifier(icon);
    if (!parsed) {
      console.warn(
        `Invalid icon identifier: "${icon}" (expected "collection:name" format)`,
      );
      return null;
    }
    return iconSvgPath(parsed.collection, parsed.name);
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
