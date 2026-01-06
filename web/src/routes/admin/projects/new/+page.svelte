<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import ProjectForm from "$lib/components/admin/ProjectForm.svelte";
  import { getAdminTags, createAdminProject } from "$lib/api";
  import type {
    AdminTag,
    AdminTagWithCount,
    CreateProjectData,
  } from "$lib/admin-types";

  let tags = $state<AdminTag[]>([]);
  let loading = $state(true);

  async function loadTags() {
    try {
      const tagsWithCounts = await getAdminTags();
      tags = tagsWithCounts.map(
        (t: AdminTagWithCount): AdminTag => ({
          id: t.id,
          slug: t.slug,
          name: t.name,
          createdAt: t.createdAt,
        }),
      );
    } catch (error) {
      console.error("Failed to load tags:", error);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    loadTags();
  });

  async function handleSubmit(data: CreateProjectData) {
    await createAdminProject(data);
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
  {#if loading}
    <div class="text-center py-12 text-admin-text-muted">Loading...</div>
  {:else}
    <div class="rounded-lg border border-admin-border bg-admin-panel p-6">
      <ProjectForm
        availableTags={tags}
        onsubmit={handleSubmit}
        submitLabel="Create Project"
      />
    </div>
  {/if}
</div>
