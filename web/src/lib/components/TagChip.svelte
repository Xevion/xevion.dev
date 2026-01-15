<script lang="ts">
  import { cn } from "$lib/utils";
  import Icon from "./Icon.svelte";

  interface Props {
    name: string;
    color?: string;
    icon?: string;
    href?: string;
    class?: string;
  }

  let { name, color, icon, href, class: className }: Props = $props();

  const baseClasses =
    "inline-flex items-center gap-1.25 rounded-r-sm rounded-l-xs bg-zinc-200/80 dark:bg-zinc-700/50 px-2 sm:px-1.5 py-1 sm:py-0.75 text-sm sm:text-xs text-zinc-700 dark:text-zinc-300 border-l-3 shadow-sm";
  const linkClasses =
    "hover:bg-zinc-300/80 dark:hover:bg-zinc-600/50 transition-colors";
</script>

{#snippet iconAndName()}
  {#if icon}
    <Icon {icon} size="size-4.25 sm:size-3.75" />
  {/if}
  <span>{name}</span>
{/snippet}

{#if href}
  <a
    {href}
    class={cn(baseClasses, linkClasses, className)}
    style="border-left-color: #{color || '06b6d4'}"
  >
    {@render iconAndName()}
  </a>
{:else}
  <span
    class={cn(baseClasses, className)}
    style="border-left-color: #{color || '06b6d4'}"
  >
    {@render iconAndName()}
  </span>
{/if}
