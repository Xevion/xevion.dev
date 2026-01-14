<script lang="ts">
  import { cn } from "$lib/utils";
  import TagChip from "./TagChip.svelte";
  import type { AdminProject } from "$lib/admin-types";

  interface Props {
    project: AdminProject & {
      tags: Array<{ iconSvg?: string; name: string; color?: string }>;
      clockIconSvg?: string;
    };
    class?: string;
  }

  let { project, class: className }: Props = $props();

  // Prefer demo URL, fallback to GitHub repo
  const projectUrl = $derived(
    project.demoUrl ||
      (project.githubRepo ? `https://github.com/${project.githubRepo}` : null),
  );

  const isLink = $derived(!!projectUrl);

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffMins < 1) return "just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffHours <= 48) return "yesterday";
    if (diffDays < 30) return `${diffDays}d ago`;
    if (diffDays < 365) return `${Math.floor(diffDays / 30)}mo ago`;
    return `${Math.floor(diffDays / 365)}y ago`;
  }
</script>

<svelte:element
  this={isLink ? "a" : "div"}
  href={isLink ? projectUrl : undefined}
  target={isLink ? "_blank" : undefined}
  rel={isLink ? "noopener noreferrer" : undefined}
  class={cn(
    "flex h-44 flex-col gap-2.5 rounded-lg border border-zinc-200 dark:border-zinc-800 bg-zinc-50 dark:bg-zinc-900/50 p-3",
    isLink &&
      "group transition-all hover:border-zinc-300 dark:hover:border-zinc-700 hover:bg-zinc-100 dark:hover:bg-zinc-800/70",
    className,
  )}
>
  <div class="flex flex-col gap-1">
    <div class="flex items-start justify-between gap-2">
      <h3
        class={cn(
          "truncate font-medium text-lg sm:text-base text-zinc-900 dark:text-zinc-100",
          isLink &&
            "transition-colors group-hover:text-zinc-950 dark:group-hover:text-white",
        )}
      >
        {project.name}
      </h3>
      <span class="shrink-0 sm:text-[0.83rem] text-zinc-600 dark:text-zinc-300">
        {formatDate(project.lastActivity)}
      </span>
    </div>
    <p
      class="line-clamp-3 sm:text-sm leading-relaxed text-zinc-600 dark:text-zinc-400"
    >
      {project.shortDescription}
    </p>
  </div>

  <!-- TODO: Add link to project search with tag filtering -->
  <div class="mt-auto flex flex-row-reverse flex-wrap-reverse gap-1">
    {#each project.tags as tag (tag.name)}
      <TagChip name={tag.name} color={tag.color} iconSvg={tag.iconSvg} />
    {/each}
  </div>
</svelte:element>
