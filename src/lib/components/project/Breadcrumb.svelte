<script lang="ts">
  import { css, cx } from "styled-system/css";
  import IconArrowLeft from "~icons/lucide/arrow-left";

  interface Props {
    slug: string;
  }

  let { slug }: Props = $props();
</script>

<nav
  class={css({
    display: "inline-flex",
    alignItems: "center",
    gap: "9px",
    mb: "22px",
    fontSize: "bodySm",
  })}
>
  <a
    href="/"
    class={cx(
      "group",
      css({
        display: "inline-flex",
        alignItems: "center",
        gap: "7px",
        color: "zinc.500",
        textDecoration: "none",
        transition: "color .15s ease",
        _hover: { color: "zinc.800", _dark: { color: "zinc.200" } },
      }),
    )}
  >
    <IconArrowLeft
      class={css({
        transition: "transform .15s ease",
        _groupHover: { transform: "translateX(-2px)" },
      })}
    />
    Index
  </a>
  <span class={css({ color: "zinc.300", _dark: { color: "zinc.600" } })}>/</span
  >
  <span
    class={css({
      fontFamily: "geist",
      fontSize: "caption",
      color: "zinc.600",
      _dark: { color: "zinc.400" },
    })}
  >
    {slug}
  </span>
</nav>
