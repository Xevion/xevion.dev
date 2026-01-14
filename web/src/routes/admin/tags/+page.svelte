<script lang="ts">
  import Button from "$lib/components/admin/Button.svelte";
  import Input from "$lib/components/admin/Input.svelte";
  import Modal from "$lib/components/admin/Modal.svelte";
  import ColorPicker from "$lib/components/admin/ColorPicker.svelte";
  import IconPicker from "$lib/components/admin/IconPicker.svelte";
  import TagChip from "$lib/components/TagChip.svelte";
  import { createAdminTag, deleteAdminTag } from "$lib/api";
  import type { CreateTagData } from "$lib/admin-types";
  import type { TagWithIconAndCount } from "./+page.server";
  import IconPlus from "~icons/lucide/plus";
  import IconX from "~icons/lucide/x";
  import IconInfo from "~icons/lucide/info";
  import { invalidateAll } from "$app/navigation";
  import { getLogger } from "@logtape/logtape";

  const logger = getLogger(["admin", "tags"]);

  interface Props {
    data: {
      tags: TagWithIconAndCount[];
    };
  }

  let { data }: Props = $props();

  // Create form state
  let showCreateForm = $state(false);
  let createName = $state("");
  let createSlug = $state("");
  let createIcon = $state<string>("");
  let createColor = $state<string | undefined>(undefined);
  let creating = $state(false);

  // Delete mode state (activated by holding Shift)
  let deleteMode = $state(false);

  // Track shift key
  $effect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Shift") deleteMode = true;
    };
    const handleKeyUp = (e: KeyboardEvent) => {
      if (e.key === "Shift") deleteMode = false;
    };
    const handleBlur = () => {
      // Reset delete mode if window loses focus
      deleteMode = false;
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);
    window.addEventListener("blur", handleBlur);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
      window.removeEventListener("blur", handleBlur);
    };
  });

  // Delete state
  let deleteModalOpen = $state(false);
  let deleteTarget = $state<TagWithIconAndCount | null>(null);
  let deleteConfirmReady = $state(false);
  let deleteTimeout: ReturnType<typeof setTimeout> | null = null;

  async function handleCreate() {
    if (!createName.trim()) return;

    creating = true;
    try {
      const createData: CreateTagData = {
        name: createName,
        slug: createSlug || undefined,
        icon: createIcon || undefined,
        color: createColor,
      };
      await createAdminTag(createData);
      await invalidateAll();
      createName = "";
      createSlug = "";
      createIcon = "";
      createColor = undefined;
      showCreateForm = false;
    } catch (error) {
      logger.error("Failed to create tag", {
        error: error instanceof Error ? error.message : String(error),
      });
      alert("Failed to create tag");
    } finally {
      creating = false;
    }
  }

  function handleTagClick(tag: TagWithIconAndCount, event: MouseEvent) {
    if (deleteMode) {
      event.preventDefault();
      event.stopPropagation();
      initiateDelete(tag);
    }
    // Otherwise, let the link navigate normally
  }

  function initiateDelete(tag: TagWithIconAndCount) {
    deleteTarget = tag;
    deleteConfirmReady = false;

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
      await deleteAdminTag(deleteTarget.id);
      await invalidateAll();
      deleteModalOpen = false;
      deleteTarget = null;
      deleteConfirmReady = false;
    } catch (error) {
      logger.error("Failed to delete tag", {
        error: error instanceof Error ? error.message : String(error),
      });
      alert("Failed to delete tag");
    }
  }
</script>

<svelte:head>
  <title>Tags | Admin</title>
</svelte:head>

<div class="space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <div class="flex items-center gap-2">
        <h1 class="text-xl font-semibold text-admin-text">Tags</h1>
        <span
          class="text-admin-text-muted hover:text-admin-text cursor-help transition-colors"
          title="Hold Shift and click a tag to delete it"
        >
          <IconInfo class="w-4 h-4" />
        </span>
      </div>
      <p class="mt-1 text-sm text-admin-text-muted">
        Manage project tags and categories
      </p>
    </div>
    <Button
      variant="primary"
      onclick={() => (showCreateForm = !showCreateForm)}
    >
      {#if showCreateForm}
        <IconX class="w-4 h-4 mr-2" />
      {:else}
        <IconPlus class="w-4 h-4 mr-2" />
      {/if}
      {showCreateForm ? "Cancel" : "New Tag"}
    </Button>
  </div>

  <!-- Create Form -->
  {#if showCreateForm}
    <div
      class="rounded-xl border border-admin-border bg-admin-surface p-6 shadow-sm shadow-black/10 dark:shadow-black/20"
    >
      <h3 class="text-base font-medium text-admin-text mb-4">Create New Tag</h3>
      <div class="grid gap-4 md:grid-cols-2">
        <Input
          label="Name"
          type="text"
          bind:value={createName}
          placeholder="TypeScript"
          required
        />
        <Input
          label="Slug"
          type="text"
          bind:value={createSlug}
          placeholder="Leave empty to auto-generate"
        />
      </div>
      <div class="mt-4">
        <IconPicker bind:selectedIcon={createIcon} label="Icon (optional)" />
      </div>
      <div class="mt-4">
        <ColorPicker bind:selectedColor={createColor} />
      </div>
      <div class="mt-4 flex justify-end gap-2">
        <Button variant="secondary" onclick={() => (showCreateForm = false)}>
          Cancel
        </Button>
        <Button
          variant="primary"
          onclick={handleCreate}
          disabled={creating || !createName.trim()}
        >
          {creating ? "Creating..." : "Create Tag"}
        </Button>
      </div>
    </div>
  {/if}

  <!-- Tags Grid -->
  {#if data.tags.length === 0}
    <div class="text-center py-12">
      <p class="text-admin-text-muted mb-4">No tags yet</p>
      <Button variant="primary" onclick={() => (showCreateForm = true)}>
        Create your first tag
      </Button>
    </div>
  {:else}
    <div class="space-y-3">
      <!-- Delete mode indicator -->
      <div
        class="h-6 flex items-center transition-opacity duration-150"
        class:opacity-100={deleteMode}
        class:opacity-0={!deleteMode}
      >
        <span
          class="text-sm text-red-500 dark:text-red-400 font-medium flex items-center gap-1.5"
        >
          <IconX class="w-4 h-4" />
          Click a tag to delete it
        </span>
      </div>

      <!-- Tags -->
      <div class="flex flex-wrap gap-2 max-w-3xl">
        {#each data.tags as tag (tag.id)}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div onclick={(e) => handleTagClick(tag, e)} class="contents">
            <TagChip
              name={tag.name}
              color={deleteMode ? "ef4444" : tag.color}
              iconSvg={tag.iconSvg}
              href={`/admin/tags/${tag.slug}`}
              class="transition-all duration-150 {deleteMode
                ? 'bg-red-100/80 dark:bg-red-900/40 cursor-pointer'
                : ''}"
            />
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<!-- Delete Confirmation Modal -->
<Modal
  bind:open={deleteModalOpen}
  title="Delete Tag"
  description="Are you sure you want to delete this tag? This will remove it from all projects."
  confirmText={deleteConfirmReady ? "Delete" : "Wait 2s..."}
  confirmVariant="danger"
  onconfirm={confirmDelete}
  oncancel={cancelDelete}
>
  {#if deleteTarget}
    <div
      class="rounded-md bg-admin-surface-hover/50 border border-admin-border p-3"
    >
      <p class="font-medium text-admin-text">{deleteTarget.name}</p>
      <p class="text-sm text-admin-text-secondary">
        Used in {deleteTarget.projectCount} project{deleteTarget.projectCount ===
        1
          ? ""
          : "s"}
      </p>
    </div>
  {/if}
</Modal>
