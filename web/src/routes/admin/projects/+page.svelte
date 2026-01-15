<script lang="ts">
  import Button from "$lib/components/admin/Button.svelte";
  import Table from "$lib/components/admin/Table.svelte";
  import TagChip from "$lib/components/TagChip.svelte";
  import { goto } from "$app/navigation";
  import type { ProjectWithTagIcons } from "./+page.server";
  import type { ProjectStatus } from "$lib/admin-types";
  import IconPlus from "~icons/lucide/plus";

  // Status display configuration (colors match Badge component)
  const STATUS_CONFIG: Record<ProjectStatus, { color: string; label: string }> =
    {
      active: { color: "10b981", label: "Active" },
      maintained: { color: "6366f1", label: "Maintained" },
      archived: { color: "71717a", label: "Archived" },
      hidden: { color: "52525b", label: "Hidden" },
    };

  interface Props {
    data: {
      projects: ProjectWithTagIcons[];
    };
  }

  let { data }: Props = $props();

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    const timeStr = date.toLocaleTimeString("en-US", {
      hour: "numeric",
      minute: "2-digit",
      hour12: true,
    });

    // Recent: relative timestamps
    if (diffMins < 1) return "just now";
    if (diffMins === 1) return "1 minute ago";
    if (diffMins < 60) return `${diffMins} minutes ago`;
    if (diffHours === 1) return "1 hour ago";
    if (diffHours < 24) return `${diffHours} hours ago`;

    // Yesterday: relative with time
    if (diffDays === 1 || (diffHours >= 24 && diffHours < 48)) {
      return `Yesterday at ${timeStr}`;
    }

    // Older: absolute timestamp with time
    const dateOptions: Intl.DateTimeFormatOptions = {
      month: "short",
      day: "numeric",
    };
    // Only show year if different from current year
    if (date.getFullYear() !== now.getFullYear()) {
      dateOptions.year = "numeric";
    }
    const datePartStr = date.toLocaleDateString("en-US", dateOptions);
    return `${datePartStr} at ${timeStr}`;
  }
</script>

<svelte:head>
  <title>Projects | Admin</title>
</svelte:head>

<div class="space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-xl font-semibold text-admin-text">Projects</h1>
      <p class="mt-1 text-sm text-admin-text-muted">
        Manage your project portfolio
      </p>
    </div>
    <Button variant="primary" href="/admin/projects/new">
      <IconPlus class="w-4 h-4 mr-2" />
      New Project
    </Button>
  </div>

  <!-- Projects Table -->
  {#if data.projects.length === 0}
    <div class="text-center py-12">
      <p class="text-admin-text-muted mb-4">No projects yet</p>
      <Button variant="primary" href="/admin/projects/new"
        >Create your first project</Button
      >
    </div>
  {:else}
    <Table>
      <thead class="bg-admin-surface-hover">
        <tr>
          <th
            class="px-4 py-3 text-left text-xs font-medium text-admin-text-muted"
          >
            Name
          </th>
          <th
            class="px-4 py-3 text-left text-xs font-medium text-admin-text-muted"
          >
            Status
          </th>
          <th
            class="px-4 py-3 text-left text-xs font-medium text-admin-text-muted"
          >
            Tags
          </th>
          <th
            class="px-4 py-3 text-left text-xs font-medium text-admin-text-muted"
          >
            Last Activity
          </th>
        </tr>
      </thead>
      <tbody class="divide-y divide-admin-border">
        {#each data.projects as project (project.id)}
          <tr
            class="hover:bg-admin-surface-hover/50 transition-colors cursor-pointer"
            onclick={() => goto(`/admin/projects/${project.id}`)}
            onkeydown={(e) =>
              (e.key === "Enter" || e.key === " ") &&
              goto(`/admin/projects/${project.id}`)}
            role="link"
            tabindex="0"
          >
            <td class="px-4 py-3">
              <div class="flex items-center gap-3">
                <div>
                  <div class="font-medium text-admin-text">
                    {project.name}
                  </div>
                  <div class="text-xs text-admin-text-muted">
                    {project.slug}
                  </div>
                </div>
              </div>
            </td>
            <td class="px-4 py-3">
              <TagChip
                name={STATUS_CONFIG[project.status].label}
                color={STATUS_CONFIG[project.status].color}
              />
            </td>
            <td class="px-4 py-3">
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="flex flex-wrap gap-1"
                onclick={(e) => e.stopPropagation()}
                onkeydown={(e) => e.stopPropagation()}
              >
                {#each project.tags.slice(0, 3) as tag (tag.id)}
                  <TagChip
                    name={tag.name}
                    color={tag.color}
                    iconSvg={tag.iconSvg}
                    href={`/admin/tags/${tag.slug}`}
                  />
                {/each}
                {#if project.tags.length > 3}
                  <span
                    class="inline-flex items-center px-2 py-1 text-xs text-admin-text-muted bg-admin-surface-hover rounded"
                  >
                    +{project.tags.length - 3}
                  </span>
                {/if}
              </div>
            </td>
            <td class="px-4 py-3 text-admin-text-secondary text-sm">
              {formatDate(project.lastActivity)}
            </td>
          </tr>
        {/each}
      </tbody>
    </Table>
  {/if}
</div>
