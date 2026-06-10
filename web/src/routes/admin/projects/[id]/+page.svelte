<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import ProjectForm from "$lib/components/admin/ProjectForm.svelte";
  import Modal from "$lib/components/admin/Modal.svelte";
  import { updateAdminProject, deleteAdminProject } from "$lib/api";
  import { ApiError } from "$lib/errors";
  import { err } from "true-myth/result";
  import type { UpdateProjectData, CreateProjectData } from "$lib/admin-types";
  import type { PageData } from "./$types";
  import { getLogger } from "@logtape/logtape";
  import { toast } from "$lib/toast";
  import { css } from "styled-system/css";
  import { pageDescriptionClass, adminCardClass } from "$lib/styles/admin";

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
    const result = await deleteAdminProject(data.project.id);
    if (result.isErr) {
      logger.error("Failed to delete project", { error: result.error });
      toast.error(result.error.message);
      return;
    }
    goto(resolve("/admin/projects"));
  }

  async function handleSubmit(formData: CreateProjectData) {
    if (!data.project) {
      return err(new ApiError(404, "Not Found", "Project not found"));
    }

    const updateData: UpdateProjectData = {
      ...formData,
      id: data.project.id,
    };
    const result = await updateAdminProject(updateData);
    if (result.isOk) {
      goto(resolve("/admin/projects"));
    }
    return result;
  }
</script>

<svelte:head>
  <title>Edit Project | Admin</title>
</svelte:head>

<div class={css({ maxW: "48rem", spaceY: "6" })}>
  <!-- Header -->
  <div>
    <h1
      class={css({ fontSize: "2xl", fontWeight: "bold", color: "admin.text" })}
    >
      Edit Project
    </h1>
    <p class={pageDescriptionClass}>Update project details and settings</p>
  </div>

  <!-- Form -->
  {#if !data.project}
    <div class={css({ textAlign: "center", py: "12" })}>
      <p class={css({ color: "admin.textMuted", mb: "4" })}>
        Project not found
      </p>
      <a
        href={resolve("/admin/projects")}
        class={css({
          color: "admin.accent",
          _hover: { color: "admin.accentHover" },
        })}
      >
        Back to projects
      </a>
    </div>
  {:else}
    <div class={adminCardClass}>
      <!-- Key on project id so switching projects fully remounts the form. The
           ContentEditor snapshots its content at mount (see ProjectForm), so a
           reused instance would keep the previous project's detail document. -->
      {#key data.project.id}
        <ProjectForm
          project={data.project}
          availableTags={data.availableTags}
          onsubmit={handleSubmit}
          ondelete={initiateDelete}
          submitLabel="Update Project"
        />
      {/key}
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
      class={css({
        rounded: "md",
        bg: "admin.surfaceHover/50",
        borderWidth: "1px",
        borderColor: "admin.border",
        p: "3",
      })}
    >
      <p class={css({ fontWeight: "medium", color: "admin.text" })}>
        {data.project.name}
      </p>
      <p class={css({ fontSize: "sm", color: "admin.textSecondary" })}>
        {data.project.slug}
      </p>
    </div>
  {/if}
</Modal>
