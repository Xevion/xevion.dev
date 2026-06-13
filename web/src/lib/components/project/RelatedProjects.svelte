<script lang="ts">
  import { css } from "styled-system/css";
  import IconChevronRight from "~icons/lucide/chevron-right";
  import type { ApiRelatedProject } from "$lib/bindings";
  import ProjectCover from "$lib/components/ProjectCover.svelte";
  import { resolveAccent } from "$lib/project-display";
  import { morph } from "$lib/stores/morph.svelte";

  // Curated "related work" — a 2-column list (not cards), authored order. Meta is
  // the project type only (the cross-language "lang" hint was dropped by design).
  interface Props {
    related: ApiRelatedProject[];
    onOpen?: (r: ApiRelatedProject) => void;
  }

  let { related, onOpen }: Props = $props();

  function go(r: ApiRelatedProject) {
    morph.slug = r.slug;
    onOpen?.(r);
  }
</script>

<div
  class={css({
    mt: "30px",
    pt: "26px",
    borderTopWidth: "1px",
    borderColor: "border.hairline",
  })}
>
  <h2 class={css({ textStyle: "label.eyebrow", m: "0 0 14px" })}>
    Related work
  </h2>
  <div class="rd-related-grid">
    {#each related as r (r.slug)}
      <a
        href="/projects/{r.slug}"
        onclick={() => go(r)}
        class={css({
          display: "flex",
          alignItems: "center",
          gap: "13px",
          p: "12px 14px",
          rounded: "11px",
          borderWidth: "1px",
          borderColor: "zinc.200",
          bg: "surface",
          textDecoration: "none",
          transition: "border-color .16s ease, transform .16s ease",
          _hover: { borderColor: "zinc.300", transform: "translateY(-1px)" },
          _dark: { borderColor: "zinc.800" },
        })}
      >
        <div
          class={css({
            w: "50px",
            h: "50px",
            rounded: "9px",
            overflow: "hidden",
            flexShrink: "0",
            bg: "surface.secondary",
            borderWidth: "1px",
            borderColor: "zinc.100",
            _dark: { borderColor: "zinc.800" },
          })}
        >
          <ProjectCover
            seed={r.name}
            accent={resolveAccent(r.accentColor)}
            cols={6}
            rows={6}
            cell={10}
          />
        </div>
        <div class={css({ minW: "0", flex: "1" })}>
          <h4
            class={css({
              fontSize: "15px",
              fontWeight: "600",
              color: "zinc.900",
              _dark: { color: "zinc.50" },
            })}
          >
            {r.name}
          </h4>
          {#if r.projectType}
            <div
              class={css({
                mt: "3px",
                fontFamily: "geist",
                fontSize: "meta",
                color: "zinc.400",
              })}
            >
              {r.projectType}
            </div>
          {/if}
        </div>
        <IconChevronRight
          class={css({
            w: "14px",
            h: "14px",
            flexShrink: "0",
            color: "zinc.300",
          })}
        />
      </a>
    {/each}
  </div>
</div>

<style>
  .rd-related-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
  @media (max-width: 640px) {
    .rd-related-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
