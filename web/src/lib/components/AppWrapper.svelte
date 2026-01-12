<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";
  import ThemeToggle from "./ThemeToggle.svelte";

  let {
    class: className = "",
    bgColor = "",
    showThemeToggle = true,
    children,
  }: {
    class?: string;
    bgColor?: string;
    showThemeToggle?: boolean;
    children?: Snippet;
  } = $props();
</script>

<!--
  Background: Public pages get their background from root +layout.svelte for persistence.
  Admin/internal pages can use bgColor prop to set their own background.
-->
{#if bgColor}
  <div
    class={cn(
      "pointer-events-none fixed inset-0 -z-20 transition-colors duration-300",
      bgColor,
    )}
  ></div>
{/if}

<main
  class={cn(
    "relative min-h-screen text-zinc-900 dark:text-zinc-50 transition-colors duration-300",
    className,
  )}
>
  {#if showThemeToggle}
    <div class="absolute top-5 right-6 z-50">
      <ThemeToggle />
    </div>
  {/if}
  {#if children}
    {@render children()}
  {/if}
</main>
