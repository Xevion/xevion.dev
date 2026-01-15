<script lang="ts">
  interface Props {
    icon: string; // "collection:name" format, e.g., "lucide:home"
    class?: string;
    size?: string; // Tailwind size class, e.g., "size-4"
  }

  let { icon, class: className = "", size = "size-4" }: Props = $props();

  let svg = $state<string | null>(null);
  let loading = $state(true);
  let error = $state(false);

  // Validate and parse icon identifier into collection and name
  const iconParts = $derived.by(() => {
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
    return {
      collection: icon.slice(0, colonIndex),
      name: icon.slice(colonIndex + 1),
    };
  });

  // Fetch icon when identifier changes
  $effect(() => {
    const parts = iconParts;
    if (!parts) {
      error = true;
      loading = false;
      return;
    }

    const url = `/api/icons/${parts.collection}/${parts.name}.svg`;
    loading = true;
    error = false;
    svg = null;

    fetch(url)
      .then((res) => {
        if (!res.ok) throw new Error(`Icon not found: ${icon}`);
        return res.text();
      })
      .then((svgText) => {
        svg = svgText;
        loading = false;
      })
      .catch(() => {
        error = true;
        loading = false;
      });
  });
</script>

{#if loading}
  <!-- Shimmer placeholder - reserves space to prevent layout shift -->
  <span
    class="inline-block {size} animate-pulse rounded bg-zinc-200 dark:bg-zinc-700"
    aria-hidden="true"
  ></span>
{:else if error}
  <!-- Error fallback - subtle empty indicator -->
  <span class="inline-block {size} rounded opacity-30" aria-hidden="true"
  ></span>
{:else if svg}
  <!-- Render SVG inline - [&>svg]:size-full makes SVG fill container -->
  <span
    class="inline-flex items-center justify-center {size} {className} [&>svg]:size-full"
    aria-hidden="true"
  >
    <!-- eslint-disable-next-line svelte/no-at-html-tags -- SVG from our API (trusted @iconify/json) -->
    {@html svg}
  </span>
{/if}
