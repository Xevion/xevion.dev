<script lang="ts">
  import TagList from "$lib/components/TagList.svelte";
  import { telemetry } from "$lib/telemetry";
  import type { PageData } from "./$types";
  import { css, cx } from "styled-system/css";
  import { flex, wrap } from "styled-system/patterns";

  let { data }: { data: PageData } = $props();
  const project = $derived(data.project);

  function trackLink(url: string) {
    telemetry.trackExternalLink(url, "project");
  }

  const linkBtnClass = flex({
    align: "center",
    columnGap: "1.5",
    px: "2.5",
    py: "1",
    rounded: "sm",
    fontSize: "sm",
    bg: "zinc.100",
    color: "zinc.800",
    shadow: "sm",
    transition: "colors",
    _dark: { bg: "zinc.900", color: "zinc.100" },
    _hover: { bg: "zinc.200", _dark: { bg: "zinc.800" } },
  });

  const proseClass = css({
    color: "zinc.700",
    _dark: { color: "zinc.200" },
    // Body headings are h2–h4 (editor schema); the page header owns the sole h1.
    "& h2": {
      fontSize: "xl",
      fontWeight: "bold",
      color: "zinc.900",
      mt: "6",
      mb: "2",
      _dark: { color: "white" },
    },
    "& h3": {
      fontSize: "lg",
      fontWeight: "semibold",
      color: "zinc.900",
      mt: "4",
      mb: "1.5",
      _dark: { color: "zinc.100" },
    },
    "& p": { my: "3", lineHeight: "relaxed" },
    "& ul": { listStyle: "disc", pl: "6", my: "3" },
    "& ol": { listStyle: "decimal", pl: "6", my: "3" },
    "& li": { my: "1" },
    "& a": {
      color: "blue.600",
      textDecoration: "underline",
      _dark: { color: "blue.400" },
    },
    "& blockquote": {
      borderLeftWidth: "3px",
      borderColor: "zinc.300",
      pl: "4",
      color: "zinc.600",
      fontStyle: "italic",
      my: "4",
      _dark: { borderColor: "zinc.700", color: "zinc.400" },
    },
    "& code": {
      bg: "zinc.100",
      px: "1.5",
      py: "0.5",
      rounded: "sm",
      fontFamily: "mono",
      fontSize: "0.85em",
      _dark: { bg: "zinc.800" },
    },
    // Code blocks (Shiki `pre.shiki`) are styled in the component style block —
    // overriding Shiki's inline canvas needs plain CSS with !important, which
    // Panda css() can't express. Inline `& code` above is reset there too.
    "& hr": {
      borderColor: "zinc.200",
      my: "6",
      _dark: { borderColor: "zinc.700" },
    },
    "& img": { maxW: "full", rounded: "md", my: "4" },
  });
</script>

<svelte:head>
  <title>{project.name} | Xevion</title>
  <meta name="description" content={project.shortDescription} />
</svelte:head>

<main
  class={cx(
    "page-main",
    css({ overflowX: "hidden", fontFamily: "schibsted", pb: "16" }),
  )}
>
  <div class={flex({ direction: "column", align: "center", pt: "14" })}>
    <article
      class={css({ maxW: "42rem", w: "full", mx: "4", sm: { mx: "6" } })}
    >
      <a
        href="/"
        class={css({
          fontSize: "sm",
          color: "zinc.500",
          transition: "colors",
          _hover: { color: "zinc.800", _dark: { color: "zinc.200" } },
        })}
      >
        ← Back
      </a>

      <header
        class={css({
          mt: "4",
          pb: "5",
          borderBottomWidth: "1px",
          borderColor: "zinc.200",
          _dark: { borderColor: "zinc.700" },
        })}
      >
        <h1
          class={css({
            fontSize: "3xl",
            fontWeight: "bold",
            color: "zinc.900",
            _dark: { color: "white" },
          })}
        >
          {project.name}
        </h1>
        <p
          class={css({
            mt: "2",
            fontSize: "lg",
            color: "zinc.600",
            _dark: { color: "zinc.400" },
          })}
        >
          {project.shortDescription}
        </p>

        {#if project.tags.length > 0}
          <TagList tags={project.tags} class={css({ mt: "3" })} />
        {/if}

        {#if project.links.length > 0}
          <div class={wrap({ gap: "2", mt: "4" })}>
            {#each project.links as link (link.url)}
              <a
                href={link.url}
                target="_blank"
                rel="noopener noreferrer"
                onclick={() => trackLink(link.url)}
                class={linkBtnClass}
              >
                {link.title ?? link.url}
              </a>
            {/each}
          </div>
        {/if}
      </header>

      <div class={cx("project-detail", proseClass, css({ mt: "5" }))}>
        <!-- eslint-disable-next-line svelte/no-at-html-tags -- server-rendered, sanitized TipTap output -->
        {@html data.html}
      </div>
    </article>
  </div>
</main>

<style>
  /* Code blocks (Shiki output). Shiki emits light-theme token colors inline plus a
     --shiki-dark custom property per token; the dark palette is activated under an
     .dark ancestor (the site's class-based dark mode lives on <html>).

     The canvas is overridden to github's subtle-canvas so the block stays distinct
     from the page — Shiki's own light canvas is pure white, invisible on a white
     page. Canvas/scrollbar colors come from the shared --code-canvas/--code-scrollbar
     vars (panda.config.ts globalCss), which flip under .dark on their own, so only
     the token color needs a dark rule below. The canvas override needs !important to
     beat Shiki's inline background. */
  :global(.project-detail .shiki) {
    margin: 1rem 0;
    padding: 0.875rem 1rem;
    border-radius: 0.5rem;
    font-size: 0.85rem;
    line-height: 1.5;
    max-height: 32rem;
    overflow: auto;
    background-color: var(--code-canvas) !important;
    scrollbar-width: thin;
    scrollbar-color: var(--code-scrollbar) transparent;
  }
  :global(.project-detail .shiki code) {
    background: none;
    padding: 0;
    font-size: inherit;
    border-radius: 0;
  }
  :global(.project-detail .shiki::-webkit-scrollbar) {
    width: 0.5rem;
    height: 0.5rem;
  }
  :global(.project-detail .shiki::-webkit-scrollbar-track) {
    background: transparent;
  }
  :global(.project-detail .shiki::-webkit-scrollbar-thumb) {
    background-color: var(--code-scrollbar);
    border-radius: 0.25rem;
  }
  /* Token colors only — canvas/scrollbar flip via the shared --code-* vars. */
  :global(.dark .project-detail .shiki),
  :global(.dark .project-detail .shiki span) {
    color: var(--shiki-dark) !important;
  }
</style>
