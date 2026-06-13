<script lang="ts">
  import { telemetry } from "$lib/telemetry";
  import { morph } from "$lib/stores/morph.svelte";
  import { resolveAccent, readableInk } from "$lib/project-display";
  import ProjectHero from "$lib/components/project/ProjectHero.svelte";
  import ProjectMetaRail from "$lib/components/project/ProjectMetaRail.svelte";
  import ProjectGallery from "$lib/components/project/ProjectGallery.svelte";
  import RelatedProjects from "$lib/components/project/RelatedProjects.svelte";
  import Breadcrumb from "$lib/components/project/Breadcrumb.svelte";
  import ProjectDetailHeader from "$lib/components/project/ProjectDetailHeader.svelte";
  import type { PageData } from "./$types";
  import type { ApiRelatedProject } from "$lib/bindings";
  import { css, cx } from "styled-system/css";
  import { prose } from "styled-system/recipes";

  let { data }: { data: PageData } = $props();
  const project = $derived(data.project);

  // Author-set accent flows to descendants via the --accent CSS var (set below);
  // --accent-ink is the legible text color for solid-accent fills.
  const accent = $derived(resolveAccent(project.accentColor));
  const accentInk = $derived(readableInk(accent));
  // The Gallery §-heading continues the prose section numbering.
  const galleryN = $derived(data.sectionCount + 1);

  // Mark this project as the morph target so back-navigation reverses into its card.
  $effect(() => {
    morph.slug = project.slug;
  });

  function trackLink(url: string) {
    telemetry.trackExternalLink(url, "project");
  }
  function openRelated(r: ApiRelatedProject) {
    telemetry.track({
      name: "project_interaction",
      properties: {
        action: "detail_view",
        projectSlug: r.slug,
        projectName: r.name,
      },
    });
  }
</script>

<svelte:head>
  <title>{project.name} | Xevion</title>
  <meta name="description" content={project.shortDescription} />
</svelte:head>

<main
  class={cx(
    "page-main",
    css({ overflowX: "hidden", fontFamily: "schibsted", pb: "20" }),
  )}
>
  <div class={css({ display: "flex", justifyContent: "center", pt: "14" })}>
    <div
      class={css({
        maxW: "940px",
        w: "full",
        px: "40px",
        "@media (max-width: 760px)": { px: "16px" },
      })}
      style="--accent: {accent}; --accent-ink: {accentInk}"
    >
      <Breadcrumb slug={project.slug} />

      <ProjectDetailHeader {project} now={data.now} />

      <ProjectHero {project} />

      <div class="rd-detail-grid">
        <div class={css({ minW: "0" })}>
          {#if data.html}
            <div class={cx("project-detail", prose())}>
              <!-- eslint-disable-next-line svelte/no-at-html-tags -- server-rendered, sanitized TipTap output -->
              {@html data.html}
            </div>
          {/if}

          {#if project.media.length > 0}
            <ProjectGallery media={project.media} n={galleryN} />
          {/if}
        </div>

        <ProjectMetaRail {project} now={data.now} onLink={trackLink} />
      </div>

      {#if project.related.length > 0}
        <RelatedProjects related={project.related} onOpen={openRelated} />
      {/if}
    </div>
  </div>
</main>

<style>
  /* Two-column reading layout with a sticky meta rail. */
  :global(.rd-detail-grid) {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 244px;
    gap: 48px;
    margin-top: 26px;
    align-items: start;
  }
  :global(.rd-rail) {
    position: sticky;
    top: 28px;
    padding: 18px 20px;
    border: 1px solid var(--colors-border-hairline);
    border-radius: 12px;
    background: var(--colors-surface);
  }

  @media (max-width: 760px) {
    :global(.rd-detail-grid) {
      grid-template-columns: 1fr;
      gap: 26px;
    }
    :global(.rd-rail) {
      position: static;
    }
  }

  /* Code block: bordered wrapper + optional language header. Shiki paints the
     body; its own canvas/border are neutralized so the wrapper owns the chrome. */
  :global(.project-detail .rd-codeblock) {
    margin: 18px 0;
    border: 1px solid #e4e4e7;
    border-radius: 9px;
    overflow: hidden;
    background: #fafafa;
  }
  :global(.dark .project-detail .rd-codeblock) {
    border-color: #27272a;
    background: #18181b;
  }
  :global(.project-detail .rd-codeblock-head) {
    font-size: 10.5px;
    font-family: "Geist Mono", ui-monospace, monospace;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: #a1a1aa;
    padding: 8px 14px;
    border-bottom: 1px solid #ececee;
    background: #f4f4f5;
  }
  :global(.dark .project-detail .rd-codeblock-head) {
    border-bottom-color: #27272a;
    background: #27272a;
  }
  :global(.project-detail .rd-codeblock .shiki) {
    margin: 0;
    padding: 13px 14px;
    font-size: 13px;
    line-height: 1.65;
    font-family: "Geist Mono", ui-monospace, monospace;
    overflow-x: auto;
    background: transparent !important;
    scrollbar-width: thin;
    scrollbar-color: var(--code-scrollbar) transparent;
  }
  :global(.project-detail .rd-codeblock .shiki code) {
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    color: inherit;
    white-space: pre;
  }
  :global(.project-detail .rd-codeblock .shiki::-webkit-scrollbar) {
    height: 0.5rem;
  }
  :global(.project-detail .rd-codeblock .shiki::-webkit-scrollbar-thumb) {
    background-color: var(--code-scrollbar);
    border-radius: 0.25rem;
  }
  /* Dark token colors only — Shiki emits them as a --shiki-dark custom property. */
  :global(.dark .project-detail .rd-codeblock .shiki),
  :global(.dark .project-detail .rd-codeblock .shiki span) {
    color: var(--shiki-dark) !important;
  }
</style>
