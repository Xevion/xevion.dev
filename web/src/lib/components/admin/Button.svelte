<script lang="ts">
  import { cn } from "$lib/utils";

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

  const baseStyles =
    "inline-flex items-center justify-center font-medium transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-admin-bg disabled:pointer-events-none disabled:opacity-50";

  const variantStyles = {
    primary:
      "bg-indigo-600 text-white hover:bg-indigo-500 focus-visible:ring-indigo-500 shadow-sm hover:shadow",
    secondary:
      "bg-transparent text-admin-text border border-zinc-700 hover:border-zinc-600 hover:bg-zinc-800/50 focus-visible:ring-zinc-500",
    danger:
      "bg-red-600 text-white hover:bg-red-500 focus-visible:ring-red-500 shadow-sm hover:shadow",
    ghost:
      "text-admin-text hover:bg-zinc-800/50 focus-visible:ring-zinc-500",
  };

  const sizeStyles = {
    sm: "h-8 px-3 text-sm rounded",
    md: "h-9 px-4 text-sm rounded-md",
    lg: "h-11 px-6 text-base rounded-md",
  };
</script>

{#if href}
  <a
    {href}
    class={cn(baseStyles, variantStyles[variant], sizeStyles[size], "cursor-pointer", className)}
    aria-disabled={disabled}
  >
    {@render children?.()}
  </a>
{:else}
  <button
    {type}
    {disabled}
    class={cn(baseStyles, variantStyles[variant], sizeStyles[size], className)}
    {onclick}
  >
    {@render children?.()}
  </button>
{/if}
