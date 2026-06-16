<script lang="ts">
  import { css } from "styled-system/css";
  import { statusMeta } from "$lib/project-display";
  import { timeAgo } from "$lib/time";
  import type { ApiProjectDetail } from "$lib/bindings";

  interface Props {
    project: ApiProjectDetail;
    /** Server-seeded clock so relative ages match across SSR/hydration. */
    now?: number;
  }

  let { project, now }: Props = $props();

  const status = $derived(statusMeta(project.status));

  // At-a-glance identity under the description: type · status · updated. One line
  // that works at every width; children are separated by middots inserted via CSS.
  const metaLine = css({
    display: "flex",
    flexWrap: "wrap",
    alignItems: "center",
    mt: "14px",
    fontFamily: "geist",
    fontSize: "metaLg",
    color: "zinc.500",
    _dark: { color: "zinc.400" },
    "& > * + *::before": {
      content: '"\\00b7"',
      mx: "9px",
      color: "zinc.400",
      _dark: { color: "zinc.500" },
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

  <p
    class={css({
      mt: "12px",
      fontSize: "18.5px",
      lineHeight: "1.5",
      color: "zinc.600",
      maxW: "640px",
      textWrap: "balance",
      "@media (max-width: 760px)": { fontSize: "16px" },
      _dark: { color: "zinc.400" },
    })}
  >
    {project.shortDescription}
  </p>

  <div class={metaLine}>
    {#if project.projectType}
      <span>{project.projectType}</span>
    {/if}
    <span
      class={css({ display: "inline-flex", alignItems: "center", gap: "6px" })}
    >
      <span
        class={css({ w: "6px", h: "6px", rounded: "full", flexShrink: "0" })}
        style="background: {status.color}"
      ></span>
      {status.label}
    </span>
    <span>Updated {timeAgo(project.lastActivity, now)}</span>
  </div>
</header>
