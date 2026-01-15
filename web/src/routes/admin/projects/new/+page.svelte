<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import ProjectForm from "$lib/components/admin/ProjectForm.svelte";
  import { createAdminProject } from "$lib/api";
  import type { CreateProjectData } from "$lib/admin-types";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  async function handleSubmit(formData: CreateProjectData) {
    await createAdminProject(formData);
    goto(resolve("/admin/projects"));
  }
</script>

<svelte:head>
  <title>New Project | Admin</title>
</svelte:head>

<div class="max-w-3xl space-y-6">
  <!-- Header -->
  <div>
    <h1 class="text-2xl font-bold text-admin-text">Create Project</h1>
    <p class="mt-1 text-sm text-admin-text-muted">
      Add a new project to your portfolio
    </p>
  </div>

  <!-- Form -->
  <div class="rounded-lg border border-admin-border bg-admin-surface p-6">
    <ProjectForm
      availableTags={data.availableTags}
      onsubmit={handleSubmit}
      submitLabel="Create Project"
    />
  </div>
</div>
