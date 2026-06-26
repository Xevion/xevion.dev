<script lang="ts">
  import { onMount } from "svelte";
  import { telemetry } from "$lib/telemetry";
  import { morph } from "$lib/stores/morph.svelte";
  import { resolveAccent, readableInk } from "$lib/project-display";
  import ProjectHero from "$lib/components/project/ProjectHero.svelte";
  import ProjectMetaRail from "$lib/components/project/ProjectMetaRail.svelte";
  import ProjectGallery from "$lib/components/project/ProjectGallery.svelte";
  import ProjectToc from "$lib/components/project/ProjectToc.svelte";
  import ProjectTocOverlay from "$lib/components/project/ProjectTocOverlay.svelte";
  import { createTocSpy } from "$lib/components/project/toc-spy.svelte";
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

  // Single scroll-spy drives the active heading for both the desktop rail and
  // the mobile overlay (see toc-spy.svelte.ts).
  const tocSpy = createTocSpy(() => data.toc);

  // Mark this project as the morph target so back-navigation reverses into its card.
  $effect(() => {
    morph.slug = project.slug;
  });

  // Honor a #heading fragment on initial load. SvelteKit's own fragment scroll
  // is unreliable here — the body is wrapped by OverlayScrollbars — so scroll the
  // target into view ourselves once the prose is in the DOM. scroll-margin-top on
  // the heading keeps it clear of the viewport top.
  onMount(() => {
    const hash = location.hash.slice(1);
    if (!hash) return;
    requestAnimationFrame(() => {
      document.getElementById(decodeURIComponent(hash))?.scrollIntoView();
    });
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

  // Reading column + meta rail. On mobile the rail collapses above the prose
  // (order: -1) so the actions/stack lead; the in-rail TOC self-hides there and
  // is replaced by the floating overlay.
  const detailGrid = css({
    display: "grid",
    gridTemplateColumns: "minmax(0, 1fr) 244px",
    gap: "48px",
    mt: "26px",
    alignItems: "start",
    "@media (max-width: 760px)": { gridTemplateColumns: "1fr", gap: "26px" },
  });
  const railCol = css({
    position: "sticky",
    top: "28px",
    display: "flex",
    flexDirection: "column",
    gap: "22px",
    "@media (max-width: 760px)": { position: "static", order: "-1" },
  });
</script>

<main
  class={cx(
    "page-main",
    css({ overflowX: "clip", fontFamily: "schibsted", pb: "20" }),
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

      <div class={detailGrid}>
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

        <div class={railCol}>
          <ProjectMetaRail {project} onLink={trackLink} />
          {#if data.toc.length > 1}
            <ProjectToc toc={data.toc} activeId={tocSpy.activeId} />
          {/if}
        </div>
      </div>

      {#if project.related.length > 0}
        <RelatedProjects related={project.related} onOpen={openRelated} />
      {/if}

      {#if data.toc.length > 1}
        <ProjectTocOverlay toc={data.toc} activeId={tocSpy.activeId} />
      {/if}
    </div>
  </div>
</main>

<style>
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

  /* Inline code highlighting. The prose `code` rule already supplies the box;
     these only colorize the tokens. Lang-mode (Shiki structure:"inline") emits
     classless <span style="color;--shiki-dark"> runs, so dark mode swaps to the
     --shiki-dark value — scoped to :not([class]) so it never touches the token
     spans below. The inline light color outranks the rule, hence !important. */
  :global(.dark .project-detail .rd-inline-code span:not([class])) {
    color: var(--shiki-dark) !important;
  }

  /* Token-mode: a single author-declared kind. Colors track the github-light /
     github-dark palette so `{:.fn}` matches what `{:lang}` would paint. */
  :global(.project-detail .rd-inline-code .tk-keyword) {
    color: #cf222e;
  }
  :global(.project-detail .rd-inline-code .tk-fn) {
    color: #8250df;
  }
  :global(.project-detail .rd-inline-code .tk-type) {
    color: #953800;
  }
  :global(.project-detail .rd-inline-code .tk-string) {
    color: #0a3069;
  }
  :global(.project-detail .rd-inline-code .tk-number),
  :global(.project-detail .rd-inline-code .tk-const),
  :global(.project-detail .rd-inline-code .tk-flag) {
    color: #0550ae;
  }
  :global(.project-detail .rd-inline-code .tk-var) {
    color: #1f2328;
  }
  :global(.project-detail .rd-inline-code .tk-comment) {
    color: #6e7781;
  }
  :global(.dark .project-detail .rd-inline-code .tk-keyword) {
    color: #ff7b72;
  }
  :global(.dark .project-detail .rd-inline-code .tk-fn) {
    color: #d2a8ff;
  }
  :global(.dark .project-detail .rd-inline-code .tk-type) {
    color: #ffa657;
  }
  :global(.dark .project-detail .rd-inline-code .tk-string) {
    color: #a5d6ff;
  }
  :global(.dark .project-detail .rd-inline-code .tk-number),
  :global(.dark .project-detail .rd-inline-code .tk-const),
  :global(.dark .project-detail .rd-inline-code .tk-flag) {
    color: #79c0ff;
  }
  :global(.dark .project-detail .rd-inline-code .tk-var) {
    color: #e6edf3;
  }
  :global(.dark .project-detail .rd-inline-code .tk-comment) {
    color: #8b949e;
  }
</style>
