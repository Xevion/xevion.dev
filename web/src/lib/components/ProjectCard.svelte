<script lang="ts">
  import { cn } from "$lib/utils";
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
  const projectUrl = project.demoUrl || (project.githubRepo ? `https://github.com/${project.githubRepo}` : null);

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

{#if projectUrl}
<a
  href={projectUrl}
  target="_blank"
  rel="noopener noreferrer"
  class={cn(
    "group flex h-44 flex-col gap-2.5 rounded-lg border border-zinc-800 bg-zinc-900/50 p-3 transition-all hover:border-zinc-700 hover:bg-zinc-800/70",
    className,
  )}
>
  <div class="flex flex-col gap-1">
    <div class="flex items-start justify-between gap-2">
      <h3
        class="truncate font-medium text-lg sm:text-base text-zinc-100 transition-colors group-hover:text-white"
      >
        {project.name}
      </h3>
      <span class="shrink-0 sm:text-[0.83rem] text-zinc-300">
        {formatDate(project.updatedAt)}
      </span>
    </div>
    <p class="line-clamp-3 sm:text-sm leading-relaxed text-zinc-400">
      {project.shortDescription}
    </p>
  </div>

  <div class="mt-auto flex flex-wrap gap-1">
    {#each project.tags as tag (tag.name)}
      <!-- TODO: Add link to project search with tag filtering -->
      <span
        class="inline-flex items-center gap-1.25 rounded-r-sm rounded-l-xs bg-zinc-700/50 px-2 sm:px-1.5 py-1 sm:py-0.75 text-sm sm:text-xs text-zinc-300 border-l-3"
        style="border-left-color: #{tag.color || '06b6d4'}"
      >
        {#if tag.iconSvg}
          <span class="size-4.25 sm:size-3.75 [&>svg]:w-full [&>svg]:h-full">
            <!-- eslint-disable-next-line svelte/no-at-html-tags -->
            {@html tag.iconSvg}
          </span>
        {/if}
        <span>{tag.name}</span>
      </span>
    {/each}
  </div>
</a>
{:else}
<div
  class={cn(
    "flex h-44 flex-col gap-2.5 rounded-lg border border-zinc-800 bg-zinc-900/50 p-3",
    className,
  )}
>
  <div class="flex flex-col gap-1">
    <div class="flex items-start justify-between gap-2">
      <h3
        class="truncate font-medium text-lg sm:text-base text-zinc-100"
      >
        {project.name}
      </h3>
      <span class="shrink-0 sm:text-[0.83rem] text-zinc-300">
        {formatDate(project.updatedAt)}
      </span>
    </div>
    <p class="line-clamp-3 sm:text-sm leading-relaxed text-zinc-400">
      {project.shortDescription}
    </p>
  </div>

  <div class="mt-auto flex flex-wrap gap-1">
    {#each project.tags as tag (tag.name)}
      <span
        class="inline-flex items-center gap-1.25 rounded-r-sm rounded-l-xs bg-zinc-700/50 px-2 sm:px-1.5 py-1 sm:py-0.75 text-sm sm:text-xs text-zinc-300 border-l-3"
        style="border-left-color: #{tag.color || '06b6d4'}"
      >
        {#if tag.iconSvg}
          <span class="size-4.25 sm:size-3.75 [&>svg]:w-full [&>svg]:h-full">
            <!-- eslint-disable-next-line svelte/no-at-html-tags -->
            {@html tag.iconSvg}
          </span>
        {/if}
        <span>{tag.name}</span>
      </span>
    {/each}
  </div>
</div>
{/if}
