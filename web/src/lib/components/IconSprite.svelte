<script lang="ts" module>
  /**
   * Convert icon identifier to valid HTML ID.
   * "simple-icons:rust" → "icon-simple-icons-rust"
   */
  export function toSymbolId(identifier: string): string {
    return `icon-${identifier.replace(/:/g, "-")}`;
  }
</script>

<script lang="ts">
  interface Props {
    icons: Record<string, string>;
  }

  let { icons }: Props = $props();

  /**
   * Extract the inner content and viewBox from an SVG string.
   * Input: '<svg viewBox="0 0 24 24" ...>content</svg>'
   * Output: { viewBox: "0 0 24 24", content: "content" }
   */
  function parseSvg(svg: string): { viewBox: string; content: string } {
    // Extract viewBox attribute
    const viewBoxMatch = svg.match(/viewBox=["']([^"']+)["']/);
    const viewBox = viewBoxMatch?.[1] ?? "0 0 24 24";

    // Extract content between <svg...> and </svg>
    const contentMatch = svg.match(/<svg[^>]*>([\s\S]*)<\/svg>/);
    const content = contentMatch?.[1] ?? "";

    return { viewBox, content };
  }
</script>

<!--
  Hidden SVG sprite containing all icon definitions as symbols.
  Icons are referenced elsewhere via <use href="#icon-{identifier}" />
-->
<svg style="display: none;" aria-hidden="true">
  <defs>
    {#each Object.entries(icons) as [id, svg] (id)}
      {@const parsed = parseSvg(svg)}
      <symbol id={toSymbolId(id)} viewBox={parsed.viewBox}>
        <!-- eslint-disable-next-line svelte/no-at-html-tags -->
        {@html parsed.content}
      </symbol>
    {/each}
  </defs>
</svg>
