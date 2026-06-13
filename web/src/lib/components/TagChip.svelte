<script lang="ts">
  import { css, cx } from "styled-system/css";
  import Icon from "./Icon.svelte";

  interface Props {
    name: string;
    color?: string;
    icon?: string;
    href?: string;
    class?: string;
    /**
     * `chip` (default) is the bordered, left-accent chip used across admin.
     * `tick` is the minimal mono label + small color dot used on public cards/rows.
     */
    variant?: "chip" | "tick";
  }

  let {
    name,
    color,
    icon,
    href,
    class: className,
    variant = "chip",
  }: Props = $props();

  // Accept colors with or without a leading `#` (admin stores them bare); never
  // double-prefix. Each variant keeps its own fallback hue.
  const withHash = (c: string) => (c.startsWith("#") ? c : `#${c}`);
  const tickColor = $derived(color ? withHash(color) : "#71717a");
  const chipColor = $derived(color ? withHash(color) : "#06b6d4");

  const baseStyles = css({
    display: "inline-flex",
    alignItems: "center",
    gap: "5px",
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
      py: "3px",
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

  const tickStyles = css({
    display: "inline-flex",
    alignItems: "center",
    gap: "5px",
    fontFamily: "geist",
    fontSize: "12px",
    fontWeight: "400",
    letterSpacing: "-0.01em",
    color: "zinc.600",
    whiteSpace: "nowrap",
    _dark: { color: "zinc.400" },
  });

  const tickDotClass = css({
    w: "6px",
    h: "6px",
    rounded: "1.5px",
    flexShrink: "0",
  });
</script>

{#if variant === "tick"}
  <span class={cx(tickStyles, className)}>
    <span class={tickDotClass} style="background: {tickColor}"></span>
    {name}
  </span>
{:else if href}
  <a
    {href}
    class={cx(baseStyles, linkStyles, className)}
    style="border-left-color: {chipColor}"
  >
    {#if icon}
      <Icon {icon} sizeClass={iconSizeClass} />
    {/if}
    <span>{name}</span>
  </a>
{:else}
  <span
    class={cx(baseStyles, className)}
    style="border-left-color: {chipColor}"
  >
    {#if icon}
      <Icon {icon} sizeClass={iconSizeClass} />
    {/if}
    <span>{name}</span>
  </span>
{/if}
