<script lang="ts">
  import { css } from "styled-system/css";
  import IconLock from "~icons/lucide/lock";

  // Closed-source hero: a dashed card explaining there's no public repo, with an
  // "Ask me about it" mailto button. On mobile (compact) the button is dropped.
  interface Props {
    compact?: boolean;
    contact?: string;
  }

  let {
    compact = false,
    contact = "mailto:xevion@xevion.dev?subject=About%20your%20project",
  }: Props = $props();
</script>

<div
  class={css({
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    gap: "14px",
    p: "15px 18px",
    rounded: "12px",
    borderWidth: "1px",
    borderStyle: "dashed",
    borderColor: "zinc.300",
    bg: "surface.secondary",
    flexWrap: "wrap",
    _dark: { borderColor: "zinc.600" },
  })}
>
  <span
    class={css({
      display: "inline-flex",
      alignItems: "center",
      gap: "11px",
      fontSize: "14px",
      lineHeight: "1.45",
      color: "zinc.700",
      _dark: { color: "zinc.300" },
    })}
  >
    <IconLock
      class={css({ w: "16px", h: "16px", flexShrink: "0", color: "zinc.500" })}
    />
    <span>
      This project is <b
        class={css({
          fontWeight: "600",
          color: "zinc.800",
          _dark: { color: "zinc.100" },
        })}>closed-source</b
      >. No public repo — but I'm happy to walk through how it works.
    </span>
  </span>
  {#if !compact}
    <a
      href={contact}
      class={css({
        display: "inline-flex",
        alignItems: "center",
        gap: "7px",
        p: "8px 14px",
        rounded: "8px",
        bg: "zinc.900",
        color: "white",
        textDecoration: "none",
        fontSize: "13px",
        fontWeight: "500",
        flexShrink: "0",
        transition: "transform .14s ease",
        _hover: { transform: "translateY(-1px)" },
        _dark: { bg: "zinc.100", color: "zinc.900" },
      })}
    >
      Ask me about it
    </a>
  {/if}
</div>
