<script lang="ts">
  import { cx } from "styled-system/css";
  import { button } from "styled-system/recipes";

  interface Props {
    variant?: "primary" | "secondary" | "danger" | "ghost";
    size?: "sm" | "md" | "lg";
    type?: "button" | "submit" | "reset";
    disabled?: boolean;
    class?: string;
    href?: string;
    onclick?: () => void;
    children?: import("svelte").Snippet;
  }

  let {
    variant = "primary",
    size = "md",
    type = "button",
    disabled = false,
    class: className,
    href,
    onclick,
    children,
  }: Props = $props();
</script>

{#if href}
  <a
    {href}
    class={cx(button({ variant, size }), className)}
    aria-disabled={disabled}
  >
    {@render children?.()}
  </a>
{:else}
  <button
    {type}
    {disabled}
    class={cx(button({ variant, size }), className)}
    {onclick}
  >
    {@render children?.()}
  </button>
{/if}
