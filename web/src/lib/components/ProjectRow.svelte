<script lang="ts">
  import { css, cx } from "styled-system/css";
  import TagChip from "./TagChip.svelte";
  import ProjectCover from "./ProjectCover.svelte";
  import { morph } from "$lib/stores/morph.svelte";
  import { projectCardView, tagColor, formatAge } from "$lib/project-display";
  import { navigateToProject } from "$lib/project-nav";
  import type { ApiAdminProject } from "$lib/bindings";

  interface Props {
    project: ApiAdminProject;
    dim?: boolean;
    /** Server-seeded clock so relative ages match across SSR/hydration. */
    now?: number;
  }

  let { project, dim = false, now }: Props = $props();

  const view = $derived(projectCardView(project));
  const active = $derived(morph.slug === project.slug);
</script>

<a
  href={view.href}
  data-slug={project.slug}
  onclick={() => navigateToProject(project)}
  class={cx(
    css({
      display: "flex",
      gap: "16px",
      alignItems: "center",
      p: "15px 8px",
      cursor: "pointer",
      textDecoration: "none",
      rounded: "8px",
      borderBottomWidth: "1px",
      borderColor: "zinc.100",
      transition: "background .15s ease, opacity .18s ease",
      _hover: { bg: "surface.secondary/90" },
      _dark: { borderColor: "zinc.800" },
    }),
    dim && css({ opacity: "0.58!", _hover: { opacity: "0.82!" } }),
  )}
>
  <div
    class={css({
      position: "relative",
      w: "52px",
      h: "52px",
      flexShrink: "0",
      rounded: "8px",
      overflow: "hidden",
      bg: "surface.secondary",
      borderWidth: "1px",
      borderColor: "zinc.100",
      _dark: { borderColor: "zinc.800" },
    })}
    style={active ? "view-transition-name: project-cover" : undefined}
  >
    <ProjectCover
      seed={project.name}
      accent={view.accent}
      cols={4}
      rows={4}
      cell={16}
      monogram={project.name[0]}
    />
  </div>

  <div class={css({ flex: "1", minW: "0" })}>
    <div
      class={css({
        display: "flex",
        alignItems: "baseline",
        gap: "10px",
        flexWrap: "wrap",
      })}
    >
      <h3
        class={css({
          fontSize: "title",
          fontWeight: "600",
          color: "zinc.900",
          letterSpacing: "-0.01em",
          _dark: { color: "zinc.50" },
        })}
        style={active ? "view-transition-name: project-title" : undefined}
      >
        {project.name}
      </h3>
      <span
        class={css({
          fontFamily: "geist",
          fontSize: "metaLg",
          color: "zinc.400",
          display: "inline-flex",
          alignItems: "center",
          gap: "8px",
        })}
      >
        {#if view.typeLabel}
          <span>{view.typeLabel}</span>
          <span class={css({ color: "zinc.300" })}>·</span>
        {/if}
        {formatAge(project.lastActivity, now)}
      </span>
    </div>
    <p
      class={css({
        mt: "4px",
        fontSize: "bodySm",
        lineHeight: "1.45",
        color: "zinc.600",
        whiteSpace: "nowrap",
        overflow: "hidden",
        textOverflow: "ellipsis",
        _dark: { color: "zinc.400" },
      })}
    >
      {project.shortDescription}
    </p>
  </div>

  {#if view.tags.length > 0}
    <div
      class={css({
        flexShrink: "0",
        maxW: "250px",
        display: "none",
        sm: { display: "flex" },
        flexWrap: "wrap",
        justifyContent: "flex-end",
        alignItems: "center",
        gap: "13px",
      })}
    >
      {#each view.tags as tag (tag.id)}
        <TagChip variant="tick" name={tag.name} color={tagColor(tag)} />
      {/each}
    </div>
  {/if}
</a>
