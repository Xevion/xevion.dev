<script lang="ts">
  import { css } from "styled-system/css";

  // A §NN section heading used outside the prose flow (e.g. the Gallery heading),
  // visually matching the CSS-counter §NN markers the prose stylesheet renders on
  // its own <h2>s. `n` is the 1-indexed section number.
  interface Props {
    n: number;
    text: string;
  }

  let { n, text }: Props = $props();

  const marker = $derived("§" + String(n).padStart(2, "0"));
</script>

<div
  class={css({
    display: "flex",
    alignItems: "center",
    gap: "12px",
    m: "30px 0 12px",
  })}
>
  <span
    class={css({
      fontFamily: "geist",
      fontSize: "caption",
      fontWeight: "600",
      flexShrink: "0",
      color: "var(--accent)",
    })}
  >
    {marker}
  </span>
  <h2 class={css({ textStyle: "heading.section" })}>
    {text}
  </h2>
  <span
    class={css({
      flex: "1",
      h: "1px",
      bg: "border.hairline",
    })}
  ></span>
</div>
