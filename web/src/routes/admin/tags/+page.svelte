<script lang="ts">
  import Button from "$lib/components/admin/Button.svelte";
  import Input from "$lib/components/admin/Input.svelte";
  import Table from "$lib/components/admin/Table.svelte";
  import Modal from "$lib/components/admin/Modal.svelte";
  import ColorPicker from "$lib/components/admin/ColorPicker.svelte";
  import {
    getAdminTags,
    createAdminTag,
    updateAdminTag,
    deleteAdminTag,
  } from "$lib/api";
  import type {
    AdminTagWithCount,
    CreateTagData,
    UpdateTagData,
  } from "$lib/admin-types";
  import IconPlus from "~icons/lucide/plus";
  import IconX from "~icons/lucide/x";

  let tags = $state<AdminTagWithCount[]>([]);
  let loading = $state(true);

  // Create form state
  let showCreateForm = $state(false);
  let createName = $state("");
  let createSlug = $state("");
  let createColor = $state<string | undefined>(undefined);
  let creating = $state(false);

  // Edit state
  let editingId = $state<string | null>(null);
  let editName = $state("");
  let editSlug = $state("");
  let editColor = $state<string | undefined>(undefined);
  let updating = $state(false);

  // Delete state
  let deleteModalOpen = $state(false);
  let deleteTarget = $state<AdminTagWithCount | null>(null);
  let deleteConfirmReady = $state(false);
  let deleteTimeout: ReturnType<typeof setTimeout> | null = null;

  async function loadTags() {
    try {
      tags = await getAdminTags();
    } catch (error) {
      console.error("Failed to load tags:", error);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    loadTags();
  });

  async function handleCreate() {
    if (!createName.trim()) return;

    creating = true;
    try {
      const data: CreateTagData = {
        name: createName,
        slug: createSlug || undefined,
        color: createColor,
      };
      await createAdminTag(data);
      await loadTags();
      createName = "";
      createSlug = "";
      createColor = undefined;
      showCreateForm = false;
    } catch (error) {
      console.error("Failed to create tag:", error);
      alert("Failed to create tag");
    } finally {
      creating = false;
    }
  }

  function startEdit(tag: AdminTagWithCount) {
    editingId = tag.id;
    editName = tag.name;
    editSlug = tag.slug;
    editColor = tag.color;
  }

  function cancelEdit() {
    editingId = null;
    editName = "";
    editSlug = "";
    editColor = undefined;
  }

  async function handleUpdate() {
    if (!editingId || !editName.trim()) return;

    updating = true;
    try {
      const data: UpdateTagData = {
        id: editingId,
        name: editName,
        slug: editSlug || undefined,
        color: editColor,
      };
      await updateAdminTag(data);
      await loadTags();
      cancelEdit();
    } catch (error) {
      console.error("Failed to update tag:", error);
      alert("Failed to update tag");
    } finally {
      updating = false;
    }
  }

  function initiateDelete(tag: AdminTagWithCount) {
    deleteTarget = tag;
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
      await deleteAdminTag(deleteTarget.id);
      await loadTags();
      deleteModalOpen = false;
      deleteTarget = null;
      deleteConfirmReady = false;
    } catch (error) {
      console.error("Failed to delete tag:", error);
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
      <h1 class="text-xl font-semibold text-admin-text">Tags</h1>
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

  <!-- Tags Table -->
  {#if loading}
    <div class="text-center py-12 text-admin-text-muted">Loading tags...</div>
  {:else if tags.length === 0}
    <div class="text-center py-12">
      <p class="text-admin-text-muted mb-4">No tags yet</p>
      <Button variant="primary" onclick={() => (showCreateForm = true)}>
        Create your first tag
      </Button>
    </div>
  {:else}
    <Table>
      <thead class="bg-admin-surface/50">
        <tr>
          <th class="px-4 py-3 text-left text-xs font-medium text-admin-text-muted">
            Name
          </th>
          <th class="px-4 py-3 text-left text-xs font-medium text-admin-text-muted">
            Slug
          </th>
          <th class="px-4 py-3 text-left text-xs font-medium text-admin-text-muted">
            Color
          </th>
          <th class="px-4 py-3 text-left text-xs font-medium text-admin-text-muted">
            Projects
          </th>
          <th class="px-4 py-3 text-right text-xs font-medium text-admin-text-muted">
            Actions
          </th>
        </tr>
      </thead>
      <tbody class="divide-y divide-admin-border/50">
        {#each tags as tag (tag.id)}
          <tr class="hover:bg-admin-surface-hover/30 transition-colors">
            {#if editingId === tag.id}
              <!-- Edit mode -->
              <td class="px-4 py-3">
                <Input
                  type="text"
                  bind:value={editName}
                  placeholder="Tag name"
                />
              </td>
              <td class="px-4 py-3">
                <Input
                  type="text"
                  bind:value={editSlug}
                  placeholder="tag-slug"
                />
              </td>
              <td class="px-4 py-3">
                {#if editColor}
                  <div class="flex items-center gap-2">
                    <div
                      class="size-6 rounded border border-admin-border"
                      style="background-color: #{editColor}"
                    />
                    <span class="text-xs text-admin-text-muted">#{editColor}</span>
                  </div>
                {:else}
                  <span class="text-xs text-admin-text-muted">No color</span>
                {/if}
              </td>
              <td class="px-4 py-3 text-admin-text">
                {tag.projectCount}
              </td>
              <td class="px-4 py-3 text-right">
                <div class="flex justify-end gap-2">
                  <Button
                    variant="secondary"
                    size="sm"
                    onclick={cancelEdit}
                    disabled={updating}
                  >
                    Cancel
                  </Button>
                  <Button
                    variant="primary"
                    size="sm"
                    onclick={handleUpdate}
                    disabled={updating || !editName.trim()}
                  >
                    {updating ? "Saving..." : "Save"}
                  </Button>
                </div>
              </td>
            {:else}
              <!-- View mode -->
              <td class="px-4 py-3 font-medium text-admin-text">
                {tag.name}
              </td>
              <td class="px-4 py-3 text-admin-text-secondary">
                {tag.slug}
              </td>
              <td class="px-4 py-3">
                {#if tag.color}
                  <div class="flex items-center gap-2">
                    <div
                      class="size-6 rounded border border-admin-border"
                      style="background-color: #{tag.color}"
                    />
                    <span class="text-xs text-admin-text-muted">#{tag.color}</span>
                  </div>
                {:else}
                  <span class="text-xs text-admin-text-muted">No color</span>
                {/if}
              </td>
              <td class="px-4 py-3 text-admin-text">
                {tag.projectCount}
              </td>
              <td class="px-4 py-3 text-right">
                <div class="flex justify-end gap-2">
                  <Button
                    variant="secondary"
                    size="sm"
                    onclick={() => startEdit(tag)}
                  >
                    Edit
                  </Button>
                  <Button
                    variant="danger"
                    size="sm"
                    onclick={() => initiateDelete(tag)}
                  >
                    Delete
                  </Button>
                </div>
              </td>
            {/if}
          </tr>
        {/each}
      </tbody>
    </Table>
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
    <div class="rounded-md bg-admin-surface-hover/50 border border-admin-border p-3">
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
