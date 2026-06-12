<script lang="ts">
  import { css, cx } from "styled-system/css";
  import { telemetry } from "$lib/telemetry";
  import TagChip from "./TagChip.svelte";
  import ProjectCover from "./ProjectCover.svelte";
  import { morph } from "$lib/stores/morph.svelte";
  import {
    accentOf,
    detectLanguage,
    formatAge,
    tagColor,
  } from "$lib/project-display";
  import type { ApiAdminProject } from "$lib/bindings";

  interface Props {
    project: ApiAdminProject;
    dim?: boolean;
  }

  let { project, dim = false }: Props = $props();

  // Every project has a detail page; demo/GitHub links live in its sidebar.
  const href = $derived(`/projects/${project.slug}`);

  const accent = $derived(accentOf(project));
  const language = $derived(detectLanguage(project));
  const rowTags = $derived(
    project.tags.filter((t) => t.name !== language?.name).slice(0, 3),
  );
  const active = $derived(morph.slug === project.slug);

  function handleClick() {
    morph.slug = project.slug;
    telemetry.track({
      name: "project_interaction",
      properties: {
        action: "detail_view",
        projectSlug: project.slug,
        projectName: project.name,
      },
    });
  }
</script>

<a
  {href}
  data-slug={project.slug}
  onclick={handleClick}
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
      {accent}
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
          fontSize: "16.5px",
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
          fontSize: "11.5px",
          color: "zinc.400",
          display: "inline-flex",
          alignItems: "center",
          gap: "8px",
        })}
      >
        {#if language}
          <span
            class={css({
              display: "inline-flex",
              alignItems: "center",
              gap: "5px",
            })}
          >
            <span
              class={css({
                w: "6px",
                h: "6px",
                rounded: "full",
                flexShrink: "0",
              })}
              style="background: {language.color}"
            ></span>
            {language.name}
          </span>
          <span class={css({ color: "zinc.300" })}>·</span>
        {/if}
        {formatAge(project.lastActivity)}
      </span>
    </div>
    <p
      class={css({
        mt: "4px",
        fontSize: "13.5px",
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

  {#if rowTags.length > 0}
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
      {#each rowTags as tag (tag.id)}
        <TagChip variant="tick" name={tag.name} color={tagColor(tag)} />
      {/each}
    </div>
  {/if}
</a>
