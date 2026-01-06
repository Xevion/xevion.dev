<script lang="ts">
  import { cn } from "$lib/utils";

  interface Props {
    class?: string;
    children?: import("svelte").Snippet;
  }

  let { class: className, children }: Props = $props();
</script>

<div
  class={cn("overflow-x-auto rounded-lg border border-admin-border", className)}
>
  <table class="w-full text-sm">
    {@render children?.()}
  </table>
</div>
