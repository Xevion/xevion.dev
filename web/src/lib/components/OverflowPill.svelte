<script lang="ts">
  import { cn } from "$lib/utils";

  interface Props {
    count: number;
    hiddenTagNames: string[];
    class?: string;
  }

  let { count, hiddenTagNames, class: className }: Props = $props();

  const tooltipText = $derived(hiddenTagNames.join(", "));
</script>

<span
  class={cn(
    "inline-flex items-center rounded-sm bg-zinc-200/50 dark:bg-zinc-600/30 px-2 sm:px-1.5 py-1 sm:py-0.75 text-sm sm:text-xs text-zinc-500 dark:text-zinc-400",
    className,
  )}
  title={tooltipText}
>
  +{count}
</span>
