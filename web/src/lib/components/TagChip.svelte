<script lang="ts">
  import { css, cx } from "styled-system/css";
  import Icon from "./Icon.svelte";

  interface Props {
    name: string;
    color?: string;
    icon?: string;
    href?: string;
    class?: string;
  }

  let { name, color, icon, href, class: className }: Props = $props();

  const baseStyles = css({
    display: "inline-flex",
    alignItems: "center",
    gap: "1.25",
    roundedRight: "sm",
    roundedLeft: "xs",
    bg: "zinc.200/80",
    px: "2",
    py: "1",
    fontSize: "sm",
    color: "zinc.700",
    borderLeftWidth: "3px",
    shadow: "sm",
    sm: {
      px: "1.5",
      py: "0.75",
      fontSize: "xs",
    },
    _dark: {
      bg: "zinc.700/50",
      color: "zinc.300",
    },
  });

  const linkStyles = css({
    transition: "colors",
    _hover: {
      bg: "zinc.300/80",
    },
    _dark: {
      _hover: {
        bg: "zinc.600/50",
      },
    },
  });

  const iconSizeClass = css({
    w: "4",
    h: "4",
    sm: {
      w: "3.5",
      h: "3.5",
    },
  });
</script>

{#snippet iconAndName()}
  {#if icon}
    <Icon {icon} sizeClass={iconSizeClass} />
  {/if}
  <span>{name}</span>
{/snippet}

{#if href}
  <a
    {href}
    class={cx(baseStyles, linkStyles, className)}
    style="border-left-color: #{color || '06b6d4'}"
  >
    {@render iconAndName()}
  </a>
{:else}
  <span
    class={cx(baseStyles, className)}
    style="border-left-color: #{color || '06b6d4'}"
  >
    {@render iconAndName()}
  </span>
{/if}
