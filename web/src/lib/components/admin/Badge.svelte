<script lang="ts">
  import { cx } from "styled-system/css";
  import { badge } from "styled-system/recipes";
  import type { ProjectStatus, EventLevel } from "$lib/admin-types";

  interface Props {
    variant?: ProjectStatus | EventLevel | "default";
    class?: string;
    children?: import("svelte").Snippet;
  }

  let { variant = "default", class: className, children }: Props = $props();
</script>

<span class={cx(badge({ variant }), className)}>
  {@render children?.()}
</span>
