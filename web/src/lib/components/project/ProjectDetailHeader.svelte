<script lang="ts">
  import { css } from "styled-system/css";
  import type { ApiProjectDetail } from "$lib/bindings";

  interface Props {
    project: ApiProjectDetail;
  }

  let { project }: Props = $props();
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
      textWrap: "pretty",
      "@media (max-width: 760px)": { fontSize: "16px" },
      _dark: { color: "zinc.400" },
    })}
  >
    {project.shortDescription}
  </p>
</header>
