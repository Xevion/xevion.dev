<script lang="ts">
  import { css, cx } from "styled-system/css";
  import type { ApiTag } from "$lib/bindings";
  import { tagColor } from "$lib/project-display";
  import IconChevron from "~icons/lucide/chevron-down";

  interface Props {
    /** Every project's tags — used to compute facet frequency + colors. */
    projects: { tags: ApiTag[] }[];
    /** Currently-selected facet names. */
    selected: string[];
    /** Count of projects matching the active facets (computed by the parent). */
    matchCount: number;
    onToggle: (name: string) => void;
    onClear: () => void;
  }

  let { projects, selected, matchCount, onToggle, onClear }: Props = $props();

  // The facets pinned to the bar, by slug so an admin rename can't silently drop
  // them. Resolved to display names below (facets are name-keyed throughout).
  const PRIMARY_SLUGS = [
    "rust",
    "typescript",
    "go",
    "svelte",
    "web-app",
    "cli",
  ];

  // tag name → { count, color, slug }, derived from the project set.
  type TagInfo = { count: number; color: string; slug: string };
  const tagInfo = $derived.by<Record<string, TagInfo>>(() => {
    const info: Record<string, TagInfo> = {};
    for (const p of projects) {
      for (const t of p.tags) {
        const existing = info[t.name];
        if (existing) existing.count += 1;
        else info[t.name] = { count: 1, color: tagColor(t), slug: t.slug };
      }
    }
    return info;
  });

  const nameBySlug = $derived(
    new Map(Object.entries(tagInfo).map(([name, i]) => [i.slug, name])),
  );
  const primaryFacets = $derived(
    PRIMARY_SLUGS.map((s) => nameBySlug.get(s)).filter((n) => n !== undefined),
  );

  const moreFacets = $derived(
    Object.keys(tagInfo)
      .filter((name) => !primaryFacets.includes(name))
      .sort((a, b) => {
        const fa = tagInfo[a].count;
        const fb = tagInfo[b].count;
        return fb - fa || a.localeCompare(b);
      }),
  );

  const totalTags = $derived(Object.keys(tagInfo).length);
  const overflowActive = $derived(
    selected.filter((s) => !primaryFacets.includes(s)).length,
  );

  let open = $state(false);
  let wrap = $state<HTMLDivElement | null>(null);

  $effect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (wrap && !wrap.contains(e.target as Node)) open = false;
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  });

  const facetClass = css({
    display: "inline-flex",
    alignItems: "center",
    justifyContent: "center",
    gap: "7px",
    cursor: "pointer",
    fontFamily: "geist",
    fontWeight: "500",
    fontSize: "12px",
    letterSpacing: "-0.01em",
    px: "11px",
    py: "5px",
    rounded: "5px",
    borderWidth: "1px",
    borderColor: "zinc.200",
    bg: "surface",
    color: "zinc.700",
    whiteSpace: "nowrap",
    shadow: "0 1px 1.5px rgba(24,24,27,.04)",
    transition:
      "border-color .14s ease, color .14s ease, background .14s ease, box-shadow .14s ease",
    _hover: {
      borderColor: "zinc.300",
      color: "zinc.900",
      shadow: "0 1px 3px rgba(24,24,27,.08)",
      _dark: { color: "zinc.50" },
    },
    _dark: { borderColor: "zinc.700", color: "zinc.300" },
  });

  const facetDotClass = css({
    w: "5px",
    h: "5px",
    rounded: "1.5px",
    flexShrink: "0",
    opacity: "0.9",
  });

  // inline on-state: border/text take the tag hue, fill is the hue at 8% alpha.
  const onStyle = (name: string) => {
    const c = tagInfo[name]?.color ?? "#71717a";
    return `border-color:${c};color:${c};background:${c}14;box-shadow:none`;
  };
</script>

<div
  class={css({
    display: "flex",
    alignItems: "center",
    gap: "10px",
    mb: "16px",
  })}
>
  <div
    class={css({
      display: "flex",
      flex: "1",
      gap: "5px",
      alignItems: "stretch",
    })}
  >
    {#each primaryFacets as name (name)}
      {@const on = selected.includes(name)}
      <button
        type="button"
        class={cx(facetClass, css({ flex: "1" }))}
        style={on ? onStyle(name) : undefined}
        onclick={() => onToggle(name)}
      >
        <span class={facetDotClass} style="background:{tagInfo[name]?.color}"
        ></span>
        {name}
      </button>
    {/each}
  </div>

  <span
    class={css({
      w: "1px",
      alignSelf: "stretch",
      bg: "zinc.200",
      my: "2px",
      mx: "1px",
      _dark: { bg: "zinc.700" },
    })}
  ></span>

  <div class={css({ position: "relative" })} bind:this={wrap}>
    <button
      type="button"
      class={cx(
        css({
          display: "inline-flex",
          alignItems: "center",
          gap: "6px",
          cursor: "pointer",
          fontFamily: "geist",
          fontSize: "metaLg",
          px: "10px",
          py: "4px",
          h: "full",
          rounded: "4px",
          borderWidth: "1px",
          borderColor: "zinc.200",
          bg: "surface/70",
          color: "zinc.600",
          transition: "border-color .14s ease, color .14s ease",
          _hover: {
            borderColor: "zinc.300",
            color: "zinc.900",
            _dark: { color: "zinc.50" },
          },
          _dark: { borderColor: "zinc.700", color: "zinc.400" },
        }),
        open &&
          css({
            borderColor: "zinc.300!",
            color: "zinc.900!",
            _dark: { color: "zinc.50!" },
          }),
      )}
      onclick={() => (open = !open)}
    >
      More
      {#if overflowActive > 0}
        <span
          class={css({
            display: "inline-grid",
            placeItems: "center",
            minW: "15px",
            h: "15px",
            px: "4px",
            rounded: "full",
            bg: "zinc.900",
            color: "white",
            fontSize: "9.5px",
            fontWeight: "600",
            _dark: { bg: "zinc.100", color: "zinc.900" },
          })}
        >
          {overflowActive}
        </span>
      {/if}
      <IconChevron
        class={css({ w: "11px", h: "11px", transition: "transform .18s ease" })}
        style={open ? "transform: rotate(180deg)" : undefined}
        aria-hidden="true"
      />
    </button>

    <div
      class={cx(
        css({
          position: "absolute",
          top: "calc(100% + 7px)",
          right: "0",
          w: "320px",
          maxW: "78vw",
          p: "12px",
          rounded: "9px",
          borderWidth: "1px",
          borderColor: "zinc.200",
          bg: "surface",
          shadow: "0 14px 40px -12px rgba(24,24,27,.22)",
          zIndex: "30",
          opacity: "0",
          transform: "translateY(-6px) scale(.985)",
          transformOrigin: "top right",
          pointerEvents: "none",
          transition: "opacity .16s ease, transform .16s ease",
          _dark: { borderColor: "zinc.700" },
        }),
        open &&
          css({
            opacity: "1!",
            transform: "none!",
            pointerEvents: "auto!",
          }),
      )}
    >
      <div
        class={css({
          display: "flex",
          flexWrap: "wrap",
          gap: "5px",
          maxH: "232px",
          overflowY: "auto",
        })}
      >
        {#each moreFacets as name (name)}
          {@const on = selected.includes(name)}
          <button
            type="button"
            class={facetClass}
            style={on ? onStyle(name) : undefined}
            onclick={() => onToggle(name)}
          >
            <span
              class={facetDotClass}
              style="background:{tagInfo[name]?.color}"
            ></span>
            {name}
          </button>
        {/each}
      </div>
      <div
        class={css({
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          mt: "11px",
          pt: "10px",
          borderTopWidth: "1px",
          borderColor: "zinc.100",
          _dark: { borderColor: "zinc.800" },
        })}
      >
        <span
          class={css({
            fontFamily: "geist",
            fontSize: "meta",
            color: "zinc.400",
          })}
        >
          {selected.length > 0 ? `${matchCount} matching` : `${totalTags} tags`}
        </span>
        <button
          type="button"
          disabled={selected.length === 0}
          class={css({
            cursor: "pointer",
            fontFamily: "geist",
            fontSize: "metaLg",
            px: "10px",
            py: "4px",
            rounded: "4px",
            borderWidth: "1px",
            borderColor: "zinc.200",
            bg: "surface.secondary",
            color: "zinc.600",
            transition: "border-color .14s ease, color .14s ease",
            _hover: {
              borderColor: "zinc.300",
              color: "zinc.900",
              _dark: { color: "zinc.50" },
            },
            _disabled: { opacity: "0.45", cursor: "default" },
            _dark: { borderColor: "zinc.700", color: "zinc.400" },
          })}
          onclick={onClear}
        >
          Clear{selected.length > 0 ? ` (${selected.length})` : ""}
        </button>
      </div>
    </div>
  </div>
</div>
