<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import ProjectForm from "$lib/components/admin/ProjectForm.svelte";
  import { updateAdminProject } from "$lib/api";
  import type {
    UpdateProjectData,
    CreateProjectData,
    TagWithIcon,
  } from "$lib/admin-types";

  interface Props {
    data: {
      project: import("$lib/admin-types").AdminProject | null;
      availableTags: TagWithIcon[];
    };
  }

  let { data }: Props = $props();

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
        submitLabel="Update Project"
      />
    </div>
  {/if}
</div>
