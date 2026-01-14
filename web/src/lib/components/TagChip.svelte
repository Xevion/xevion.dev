<script lang="ts">
  import { cn } from "$lib/utils";

  interface Props {
    name: string;
    color?: string;
    iconSvg?: string;
    class?: string;
  }

  let { name, color, iconSvg, class: className }: Props = $props();
</script>

<span
  class={cn(
    "inline-flex items-center gap-1.25 rounded-r-sm rounded-l-xs bg-zinc-200/80 dark:bg-zinc-700/50 px-2 sm:px-1.5 py-1 sm:py-0.75 text-sm sm:text-xs text-zinc-700 dark:text-zinc-300 border-l-3",
    className,
  )}
  style="border-left-color: #{color || '06b6d4'}"
>
  {#if iconSvg}
    <span class="size-4.25 sm:size-3.75 [&>svg]:w-full [&>svg]:h-full">
      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
      {@html iconSvg}
    </span>
  {/if}
  <span>{name}</span>
</span>
