<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import ProjectForm from "$lib/components/admin/ProjectForm.svelte";
  import Modal from "$lib/components/admin/Modal.svelte";
  import IconSprite from "$lib/components/IconSprite.svelte";
  import { updateAdminProject, deleteAdminProject } from "$lib/api";
  import type { UpdateProjectData, CreateProjectData } from "$lib/admin-types";
  import type { PageData } from "./$types";
  import { getLogger } from "@logtape/logtape";

  const logger = getLogger(["admin", "projects", "edit"]);

  let { data }: { data: PageData } = $props();

  // Delete modal state
  let deleteModalOpen = $state(false);
  let deleteConfirmReady = $state(false);
  let deleteTimeout: ReturnType<typeof setTimeout> | null = null;

  function initiateDelete() {
    deleteConfirmReady = false;
    deleteTimeout = setTimeout(() => {
      deleteConfirmReady = true;
    }, 2000);
    deleteModalOpen = true;
  }

  function cancelDelete() {
    if (deleteTimeout) clearTimeout(deleteTimeout);
    deleteModalOpen = false;
    deleteConfirmReady = false;
  }

  async function confirmDelete() {
    if (!data.project || !deleteConfirmReady) return;
    try {
      await deleteAdminProject(data.project.id);
      goto(resolve("/admin/projects"));
    } catch (error) {
      logger.error("Failed to delete project", {
        error: error instanceof Error ? error.message : String(error),
      });
      alert("Failed to delete project");
    }
  }

  async function handleSubmit(formData: CreateProjectData) {
    if (!data.project) return;

    const updateData: UpdateProjectData = {
      ...formData,
      id: data.project.id,
    };
    await updateAdminProject(updateData);
    goto(resolve("/admin/projects"));
  }
</script>

<svelte:head>
  <title>Edit Project | Admin</title>
</svelte:head>

<IconSprite icons={data.icons} />

<div class="max-w-3xl space-y-6">
  <!-- Header -->
  <div>
    <h1 class="text-2xl font-bold text-admin-text">Edit Project</h1>
    <p class="mt-1 text-sm text-admin-text-muted">
      Update project details and settings
    </p>
  </div>

  <!-- Form -->
  {#if !data.project}
    <div class="text-center py-12">
      <p class="text-admin-text-muted mb-4">Project not found</p>
      <a
        href={resolve("/admin/projects")}
        class="text-admin-accent hover:text-admin-accent-hover"
      >
        Back to projects
      </a>
    </div>
  {:else}
    <div class="rounded-lg border border-admin-border bg-admin-surface p-6">
      <ProjectForm
        project={data.project}
        availableTags={data.availableTags}
        onsubmit={handleSubmit}
        ondelete={initiateDelete}
        submitLabel="Update Project"
      />
    </div>
  {/if}
</div>

<!-- Delete Confirmation Modal -->
<Modal
  bind:open={deleteModalOpen}
  title="Delete Project"
  description="Are you sure you want to delete this project? This action cannot be undone."
  confirmText={deleteConfirmReady ? "Delete" : "Wait 2s..."}
  confirmVariant="danger"
  onconfirm={confirmDelete}
  oncancel={cancelDelete}
>
  {#if data.project}
    <div
      class="rounded-md bg-admin-surface-hover/50 border border-admin-border p-3"
    >
      <p class="font-medium text-admin-text">{data.project.name}</p>
      <p class="text-sm text-admin-text-secondary">{data.project.slug}</p>
    </div>
  {/if}
</Modal>
