<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import ProjectForm from "$lib/components/admin/ProjectForm.svelte";
  import { createAdminProject } from "$lib/api";
  import type { CreateProjectData } from "$lib/admin-types";
  import type { PageData } from "./$types";
  import { css } from "styled-system/css";
  import { pageDescriptionClass, adminCardClass } from "$lib/styles/admin";

  let { data }: { data: PageData } = $props();

  async function handleSubmit(formData: CreateProjectData) {
    const result = await createAdminProject(formData);
    if (result.isOk) {
      goto(resolve("/admin/projects"));
    }
    return result;
  }
</script>

<svelte:head>
  <title>New Project | Admin</title>
</svelte:head>

<div class={css({ maxW: "48rem", spaceY: "6" })}>
  <!-- Header -->
  <div>
    <h1
      class={css({ fontSize: "2xl", fontWeight: "bold", color: "admin.text" })}
    >
      Create Project
    </h1>
    <p class={pageDescriptionClass}>Add a new project to your portfolio</p>
  </div>

  <!-- Form -->
  <div class={adminCardClass}>
    <ProjectForm
      availableTags={data.availableTags}
      onsubmit={handleSubmit}
      submitLabel="Create Project"
    />
  </div>
</div>
