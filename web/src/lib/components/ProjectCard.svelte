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
    /** Non-matching (filtered) — dimmed, not hidden. */
    dim?: boolean;
    /** Server-seeded clock so relative ages match across SSR/hydration. */
    now?: number;
    class?: string;
  }

  let { project, dim = false, now, class: className }: Props = $props();

  const view = $derived(projectCardView(project));
  // This card carries the shared transition name only while it's the one opening.
  const active = $derived(morph.slug === project.slug);
</script>

<a
  href={view.href}
  data-slug={project.slug}
  onclick={() => navigateToProject(project)}
  class={cx(
    "group",
    css({
      position: "relative",
      display: "flex",
      flexDirection: "column",
      rounded: "4px",
      borderWidth: "1px",
      borderColor: "zinc.200",
      bg: "surface",
      overflow: "hidden",
      cursor: "pointer",
      textDecoration: "none",
      shadow:
        "0 1px 2px rgba(24,24,27,.04), 0 2px 10px -6px rgba(24,24,27,.06)",
      transition:
        "border-color .18s ease, box-shadow .18s ease, transform .18s ease, opacity .18s ease",
      _hover: {
        borderColor: "zinc.300",
        shadow: "0 10px 28px -10px rgba(24,24,27,.16)",
        transform: "translateY(-2px)",
      },
      _dark: { borderColor: "zinc.800" },
    }),
    dim && css({ opacity: "0.58!", _hover: { opacity: "0.82!" } }),
    className,
  )}
>
  <div
    class={css({
      position: "relative",
      h: "116px",
      bg: "surface.secondary",
      borderBottomWidth: "1px",
      borderColor: "zinc.100",
      _dark: { borderColor: "zinc.800" },
    })}
    style={active ? "view-transition-name: project-cover" : undefined}
  >
    <ProjectCover
      seed={project.name}
      accent={view.accent}
      cols={11}
      rows={5}
      cell={22}
    />
  </div>

  <div
    class={css({
      p: "13px 15px 15px",
      display: "flex",
      flexDirection: "column",
      flex: "1",
    })}
  >
    <div
      class={css({
        display: "flex",
        alignItems: "baseline",
        justifyContent: "space-between",
        gap: "10px",
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
          flexShrink: "0",
          fontFamily: "geist",
          fontSize: "meta",
          color: "zinc.400",
        })}
      >
        {formatAge(project.lastActivity, now)}
      </span>
    </div>
    <p
      class={css({
        mt: "6px",
        fontSize: "bodySm",
        lineHeight: "1.5",
        color: "zinc.600",
        lineClamp: "2",
        _dark: { color: "zinc.400" },
      })}
    >
      {project.shortDescription}
    </p>
    {#if view.tags.length > 0}
      <div
        class={css({
          mt: "auto",
          pt: "13px",
          display: "flex",
          flexWrap: "wrap",
          alignItems: "center",
          gap: "14px",
        })}
      >
        {#each view.tags as tag (tag.id)}
          <TagChip variant="tick" name={tag.name} color={tagColor(tag)} />
        {/each}
      </div>
    {/if}
  </div>
</a>
