<script lang="ts">
  import Button from "$lib/components/admin/Button.svelte";
  import Table from "$lib/components/admin/Table.svelte";
  import Modal from "$lib/components/admin/Modal.svelte";
  import TagChip from "$lib/components/TagChip.svelte";
  import { deleteAdminProject } from "$lib/api";
  import { invalidateAll } from "$app/navigation";
  import type { ProjectWithTagIcons } from "./+page.server";
  import type { ProjectStatus } from "$lib/admin-types";
  import IconPlus from "~icons/lucide/plus";
  import { getLogger } from "@logtape/logtape";

  const logger = getLogger(["admin", "projects"]);

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
      statusIcons: Record<ProjectStatus, string>;
    };
  }

  let { data }: Props = $props();

  let deleteModalOpen = $state(false);
  let deleteTarget = $state<ProjectWithTagIcons | null>(null);
  let deleteConfirmReady = $state(false);
  let deleteTimeout: ReturnType<typeof setTimeout> | null = null;

  function initiateDelete(project: ProjectWithTagIcons) {
    deleteTarget = project;
    deleteConfirmReady = false;

    // Enable confirm button after delay
    deleteTimeout = setTimeout(() => {
      deleteConfirmReady = true;
    }, 2000);

    deleteModalOpen = true;
  }

  function cancelDelete() {
    if (deleteTimeout) {
      clearTimeout(deleteTimeout);
    }
    deleteModalOpen = false;
    deleteTarget = null;
    deleteConfirmReady = false;
  }

  async function confirmDelete() {
    if (!deleteTarget || !deleteConfirmReady) return;

    try {
      await deleteAdminProject(deleteTarget.id);
      await invalidateAll();
      deleteModalOpen = false;
      deleteTarget = null;
      deleteConfirmReady = false;
    } catch (error) {
      logger.error("Failed to delete project", {
        error: error instanceof Error ? error.message : String(error),
      });
      alert("Failed to delete project");
    }
  }

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    return date.toLocaleDateString("en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
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
          <th
            class="px-4 py-3 text-right text-xs font-medium text-admin-text-muted"
          >
            Actions
          </th>
        </tr>
      </thead>
      <tbody class="divide-y divide-admin-border">
        {#each data.projects as project (project.id)}
          <tr class="hover:bg-admin-surface-hover/50 transition-colors">
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
                iconSvg={data.statusIcons[project.status]}
              />
            </td>
            <td class="px-4 py-3">
              <div class="flex flex-wrap gap-1">
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
            <td class="px-4 py-3 text-right">
              <div class="flex justify-end gap-2">
                <Button
                  variant="secondary"
                  size="sm"
                  href={`/admin/projects/${project.id}`}
                >
                  Edit
                </Button>
                <Button
                  variant="danger"
                  size="sm"
                  onclick={() => initiateDelete(project)}
                >
                  Delete
                </Button>
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </Table>
  {/if}
</div>

<!-- Delete Confirmation Modal -->
<Modal
  bind:open={deleteModalOpen}
  title="Delete Project"
  description="Are you sure you want to delete this project? This action cannot be undone."
  confirmText={deleteConfirmReady ? "Delete" : `Wait ${2}s...`}
  confirmVariant="danger"
  onconfirm={confirmDelete}
  oncancel={cancelDelete}
>
  {#if deleteTarget}
    <div
      class="rounded-md bg-admin-surface-hover/50 border border-admin-border p-3"
    >
      <p class="font-medium text-admin-text">{deleteTarget.name}</p>
      <p class="text-sm text-admin-text-secondary">{deleteTarget.slug}</p>
    </div>
  {/if}
</Modal>
