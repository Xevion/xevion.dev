<script lang="ts">
  import { css, cx } from "styled-system/css";

  interface Props {
    class?: string;
    children?: import("svelte").Snippet;
  }

  let { class: className, children }: Props = $props();
</script>

<div
  class={cx(
    css({
      overflowX: "auto",
      rounded: "lg",
      borderWidth: "1px",
      borderColor: "admin.border",
      bg: "admin.surface",
    }),
    className,
  )}
>
  <table class={css({ w: "full", fontSize: "sm" })}>
    {@render children?.()}
  </table>
</div>
