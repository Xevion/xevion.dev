<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import ProjectForm from "$lib/components/admin/ProjectForm.svelte";
  import { getAdminProject, getAdminTags, updateAdminProject } from "$lib/api";
  import type {
    AdminProject,
    AdminTag,
    AdminTagWithCount,
    CreateProjectData,
    UpdateProjectData,
  } from "$lib/admin-types";

  const projectId = $derived(($page.params as { id: string }).id);

  let project = $state<AdminProject | null>(null);
  let tags = $state<AdminTag[]>([]);
  let loading = $state(true);

  async function loadData() {
    try {
      const [projectData, tagsWithCounts] = await Promise.all([
        getAdminProject(projectId),
        getAdminTags(),
      ]);

      project = projectData;
      tags = tagsWithCounts.map(
        (t: AdminTagWithCount): AdminTag => ({
          id: t.id,
          slug: t.slug,
          name: t.name,
          createdAt: t.createdAt,
        }),
      );
    } catch (error) {
      console.error("Failed to load data:", error);
      alert("Failed to load project");
      goto(resolve("/admin/projects"));
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    loadData();
  });

  async function handleSubmit(data: CreateProjectData) {
    const updateData: UpdateProjectData = {
      ...data,
      id: projectId,
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
  {#if loading}
    <div class="text-center py-12 text-admin-text-muted">Loading...</div>
  {:else if !project}
    <div class="text-center py-12">
      <p class="text-admin-text-muted mb-4">Project not found</p>
      <a
        href={resolve("/admin/projects")}
        class="text-blue-400 hover:text-blue-300"
      >
        ← Back to projects
      </a>
    </div>
  {:else}
    <div class="rounded-lg border border-admin-border bg-admin-panel p-6">
      <ProjectForm
        {project}
        availableTags={tags}
        onsubmit={handleSubmit}
        submitLabel="Update Project"
      />
    </div>
  {/if}
</div>
