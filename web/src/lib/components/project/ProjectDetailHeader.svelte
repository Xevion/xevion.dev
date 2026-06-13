<script lang="ts">
  import { css } from "styled-system/css";
  import { statusMeta, formatAge } from "$lib/project-display";
  import type { ApiProjectDetail } from "$lib/bindings";

  interface Props {
    project: ApiProjectDetail;
    /** Server-seeded clock so relative ages match across SSR/hydration. */
    now?: number;
  }

  let { project, now }: Props = $props();

  const status = $derived(statusMeta(project.status));

  // Stand-in for the meta rail on narrow viewports, where the rail collapses below.
  const mobileSummary = css({
    display: "none",
    "@media (max-width: 760px)": {
      display: "flex",
      flexWrap: "wrap",
      alignItems: "center",
      gap: "8px",
      mt: "10px",
    },
  });
</script>

<header>
  <h1
    class={css({
      fontSize: "38px",
      fontWeight: "700",
      letterSpacing: "-0.02em",
      color: "zinc.900",
      "@media (max-width: 760px)": { fontSize: "28px" },
      _dark: { color: "white" },
    })}
    style="view-transition-name: project-title"
  >
    {project.name}
  </h1>

  <div class={mobileSummary}>
    {#if project.projectType}
      <span
        class={css({
          display: "inline-flex",
          alignItems: "center",
          p: "3px 10px",
          rounded: "full",
          fontFamily: "geist",
          fontSize: "metaLg",
          color: "zinc.800",
          borderWidth: "1px",
          bg: "color-mix(in srgb, var(--accent) 8%, transparent)",
          borderColor: "color-mix(in srgb, var(--accent) 25%, transparent)",
          _dark: { color: "zinc.200" },
        })}
      >
        {project.projectType}
      </span>
    {/if}
    <span
      class={css({ textStyle: "label.status" })}
      style="color: {status.color}">{status.label}</span
    >
    <span
      class={css({
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
      mt: "12px",
      fontSize: "18.5px",
      lineHeight: "1.5",
      color: "zinc.600",
      maxW: "640px",
      textWrap: "pretty",
      "@media (max-width: 760px)": { fontSize: "16px" },
      _dark: { color: "zinc.400" },
    })}
  >
    {project.shortDescription}
  </p>
</header>
