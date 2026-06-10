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
    "& :global(h1)": {
      fontSize: "2xl",
      fontWeight: "bold",
      color: "zinc.900",
      mt: "6",
      mb: "3",
      _dark: { color: "white" },
    },
    "& :global(h2)": {
      fontSize: "xl",
      fontWeight: "bold",
      color: "zinc.900",
      mt: "6",
      mb: "2",
      _dark: { color: "white" },
    },
    "& :global(h3)": {
      fontSize: "lg",
      fontWeight: "semibold",
      color: "zinc.900",
      mt: "4",
      mb: "1.5",
      _dark: { color: "zinc.100" },
    },
    "& :global(p)": { my: "3", lineHeight: "relaxed" },
    "& :global(ul)": { listStyle: "disc", pl: "6", my: "3" },
    "& :global(ol)": { listStyle: "decimal", pl: "6", my: "3" },
    "& :global(li)": { my: "1" },
    "& :global(a)": {
      color: "blue.600",
      textDecoration: "underline",
      _dark: { color: "blue.400" },
    },
    "& :global(blockquote)": {
      borderLeftWidth: "3px",
      borderColor: "zinc.300",
      pl: "4",
      color: "zinc.600",
      fontStyle: "italic",
      my: "4",
      _dark: { borderColor: "zinc.700", color: "zinc.400" },
    },
    "& :global(code)": {
      bg: "zinc.100",
      px: "1.5",
      py: "0.5",
      rounded: "sm",
      fontFamily: "mono",
      fontSize: "0.85em",
      _dark: { bg: "zinc.800" },
    },
    "& :global(pre)": {
      bg: "zinc.100",
      p: "3",
      rounded: "md",
      overflowX: "auto",
      my: "4",
      _dark: { bg: "zinc.900" },
    },
    "& :global(pre code)": { bg: "transparent", p: "0" },
    "& :global(hr)": {
      borderColor: "zinc.200",
      my: "6",
      _dark: { borderColor: "zinc.700" },
    },
    "& :global(img)": { maxW: "full", rounded: "md", my: "4" },
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
