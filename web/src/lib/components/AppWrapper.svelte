<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";
  import Dots from "./Dots.svelte";
  import ThemeToggle from "./ThemeToggle.svelte";

  let {
    class: className = "",
    backgroundClass = "",
    bgColor = "",
    showThemeToggle = true,
    children,
  }: {
    class?: string;
    backgroundClass?: string;
    bgColor?: string;
    showThemeToggle?: boolean;
    children?: Snippet;
  } = $props();
</script>

<div
  class={cn(
    "pointer-events-none fixed inset-0 -z-20 bg-white dark:bg-black transition-colors duration-300",
    bgColor,
  )}
></div>
<Dots class={[backgroundClass]} />
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
