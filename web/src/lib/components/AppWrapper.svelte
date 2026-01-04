<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";
  import Dots from "./Dots.svelte";

  let {
    class: className = "",
    backgroundClass = "",
    children,
  }: {
    class?: string;
    backgroundClass?: string;
    children?: Snippet;
  } = $props();
</script>

<div class="pointer-events-none fixed inset-0 -z-20 bg-black"></div>
<Dots class={[backgroundClass]} />
<main class={cn("relative min-h-screen text-zinc-50", className)}>
  {#if children}
    {@render children()}
  {/if}
</main>
