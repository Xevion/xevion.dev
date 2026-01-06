<script lang="ts">
  import { cn } from "$lib/utils";
  import type { ProjectStatus, EventLevel } from "$lib/admin-types";

  interface Props {
    variant?: ProjectStatus | EventLevel | "default";
    class?: string;
    children?: import("svelte").Snippet;
  }

  let { variant = "default", class: className, children }: Props = $props();

  const baseStyles =
    "inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium";

  const variantStyles = {
    active: "bg-emerald-500/10 text-emerald-400",
    maintained: "bg-indigo-500/10 text-indigo-400",
    archived: "bg-zinc-500/10 text-zinc-400",
    hidden: "bg-zinc-500/10 text-zinc-500",
    info: "bg-teal-500/10 text-teal-400",
    warning: "bg-amber-500/10 text-amber-400",
    error: "bg-red-500/10 text-red-400",
    default: "bg-zinc-500/10 text-zinc-400",
  };
</script>

<span class={cn(baseStyles, variantStyles[variant], className)}>
  {@render children?.()}
</span>
