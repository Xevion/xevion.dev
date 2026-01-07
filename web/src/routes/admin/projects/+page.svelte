<script lang="ts">
  import Button from "$lib/components/admin/Button.svelte";
  import Table from "$lib/components/admin/Table.svelte";
  import Badge from "$lib/components/admin/Badge.svelte";
  import Modal from "$lib/components/admin/Modal.svelte";
  import { getAdminProjects, deleteAdminProject } from "$lib/api";
  import type { AdminProject } from "$lib/admin-types";
  import IconPlus from "~icons/lucide/plus";

  let projects = $state<AdminProject[]>([]);
  let loading = $state(true);
  let deleteModalOpen = $state(false);
  let deleteTarget = $state<AdminProject | null>(null);
  let deleteConfirmReady = $state(false);
  let deleteTimeout: ReturnType<typeof setTimeout> | null = null;

  async function loadProjects() {
    try {
      projects = await getAdminProjects();
    } catch (error) {
      console.error("Failed to load projects:", error);
    } finally {
      loading = false;
    }
  }

  // Load projects on mount
  $effect(() => {
    loadProjects();
  });

  function initiateDelete(project: AdminProject) {
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
      projects = projects.filter((p) => p.id !== deleteTarget!.id);
      deleteModalOpen = false;
      deleteTarget = null;
      deleteConfirmReady = false;
    } catch (error) {
      console.error("Failed to delete project:", error);
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
      <h1 class="text-xl font-semibold text-zinc-50">Projects</h1>
      <p class="mt-1 text-sm text-zinc-500">Manage your project portfolio</p>
    </div>
    <Button variant="primary" href="/admin/projects/new">
      <IconPlus class="w-4 h-4 mr-2" />
      New Project
    </Button>
  </div>

  <!-- Projects Table -->
  {#if loading}
    <div class="text-center py-12 text-zinc-500">Loading projects...</div>
  {:else if projects.length === 0}
    <div class="text-center py-12">
      <p class="text-zinc-500 mb-4">No projects yet</p>
      <Button variant="primary" href="/admin/projects/new"
        >Create your first project</Button
      >
    </div>
  {:else}
    <Table>
      <thead class="bg-zinc-900/50">
        <tr>
          <th class="px-4 py-3 text-left text-xs font-medium text-zinc-500">
            Name
          </th>
          <th class="px-4 py-3 text-left text-xs font-medium text-zinc-500">
            Status
          </th>
          <th class="px-4 py-3 text-left text-xs font-medium text-zinc-500">
            Tags
          </th>
          <th class="px-4 py-3 text-left text-xs font-medium text-zinc-500">
            Updated
          </th>
          <th class="px-4 py-3 text-right text-xs font-medium text-zinc-500">
            Actions
          </th>
        </tr>
      </thead>
      <tbody class="divide-y divide-zinc-800/50">
        {#each projects as project (project.id)}
          <tr class="hover:bg-zinc-800/30 transition-colors">
            <td class="px-4 py-3">
              <div class="flex items-center gap-3">
                <div>
                  <div class="font-medium text-zinc-200">
                    {project.name}
                  </div>
                  <div class="text-xs text-zinc-500">
                    {project.slug}
                  </div>
                </div>
              </div>
            </td>
            <td class="px-4 py-3">
              <Badge variant={project.status}>
                {project.status}
              </Badge>
            </td>
            <td class="px-4 py-3">
              <div class="flex flex-wrap gap-1">
                {#each project.tags.slice(0, 3) as tag (tag.id)}
                  <Badge variant="default">{tag.name}</Badge>
                {/each}
                {#if project.tags.length > 3}
                  <Badge variant="default">+{project.tags.length - 3}</Badge>
                {/if}
              </div>
            </td>
            <td class="px-4 py-3 text-zinc-500 text-sm">
              {formatDate(project.updatedAt)}
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
    <div class="rounded-md bg-zinc-800/50 border border-zinc-700 p-3">
      <p class="font-medium text-zinc-200">{deleteTarget.name}</p>
      <p class="text-sm text-zinc-500">{deleteTarget.slug}</p>
    </div>
  {/if}
</Modal>
