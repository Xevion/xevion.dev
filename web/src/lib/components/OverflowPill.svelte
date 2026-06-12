<script lang="ts">
  import { css, cx } from "styled-system/css";

  interface Props {
    count: number;
    hiddenTagNames: string[];
    class?: string;
  }

  let { count, hiddenTagNames, class: className }: Props = $props();

  const tooltipText = $derived(hiddenTagNames.join(", "));
</script>

<span
  class={cx(
    css({
      display: "inline-flex",
      alignItems: "center",
      rounded: "sm",
      bg: "zinc.200/50",
      px: "2",
      py: "1",
      fontSize: "sm",
      color: "zinc.500",
      sm: {
        px: "1.5",
        py: "3px",
        fontSize: "xs",
      },
      _dark: {
        bg: "zinc.600/30",
        color: "zinc.400",
      },
    }),
    className,
  )}
  title={tooltipText}
>
  +{count}
</span>
