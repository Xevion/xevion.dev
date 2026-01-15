<script lang="ts">
  import Button from "$lib/components/admin/Button.svelte";
  import Input from "$lib/components/admin/Input.svelte";
  import Modal from "$lib/components/admin/Modal.svelte";
  import ColorPicker from "$lib/components/admin/ColorPicker.svelte";
  import IconPicker from "$lib/components/admin/IconPicker.svelte";
  import TagChip from "$lib/components/TagChip.svelte";
  import IconSprite from "$lib/components/IconSprite.svelte";
  import { updateAdminTag, deleteAdminTag } from "$lib/api";
  import { goto, invalidateAll } from "$app/navigation";
  import type { PageData } from "./$types";
  import IconArrowLeft from "~icons/lucide/arrow-left";
  import IconExternalLink from "~icons/lucide/external-link";
  import { getLogger } from "@logtape/logtape";

  const logger = getLogger(["admin", "tags", "edit"]);

  let { data }: { data: PageData } = $props();

  // Form state - initialize from loaded data (intentionally captures initial values)
  // svelte-ignore state_referenced_locally
  let name = $state(data.tag.name);
  // svelte-ignore state_referenced_locally
  let slug = $state(data.tag.slug);
  // svelte-ignore state_referenced_locally
  let icon = $state(data.tag.icon ?? "");
  // svelte-ignore state_referenced_locally
  let color = $state<string | undefined>(data.tag.color);
  let saving = $state(false);

  // Preview icon SVG - starts with server-rendered, updates on icon change
  // svelte-ignore state_referenced_locally
  let previewIconSvg = $state(
    data.tag.icon ? (data.icons[data.tag.icon] ?? "") : "",
  );
  let iconLoadTimeout: ReturnType<typeof setTimeout> | null = null;

  // Watch for icon changes and fetch new preview
  $effect(() => {
    const currentIcon = icon;

    // Clear pending timeout
    if (iconLoadTimeout) {
      clearTimeout(iconLoadTimeout);
    }

    if (!currentIcon) {
      previewIconSvg = "";
      return;
    }

    // Check if icon is already in sprite
    if (data.icons[currentIcon]) {
      previewIconSvg = data.icons[currentIcon];
      return;
    }

    // Debounce icon fetching for new icons
    iconLoadTimeout = setTimeout(async () => {
      try {
        const response = await fetch(
          `/api/icons/${currentIcon.replace(":", "/")}`,
        );
        if (response.ok) {
          const iconData = await response.json();
          previewIconSvg = iconData.svg ?? "";
        }
      } catch {
        // Keep existing preview on error
      }
    }, 200);
  });

  // Delete state
  let deleteModalOpen = $state(false);
  let deleteConfirmReady = $state(false);
  let deleteTimeout: ReturnType<typeof setTimeout> | null = null;

  async function handleSave() {
    if (!name.trim()) return;

    saving = true;
    try {
      await updateAdminTag({
        id: data.tag.id,
        name: name.trim(),
        slug: slug.trim() || undefined,
        icon: icon || undefined,
        color: color,
      });

      // If slug changed, navigate to new URL
      const newSlug = slug.trim() || data.tag.slug;
      if (newSlug !== data.tag.slug) {
        await goto(`/admin/tags/${newSlug}`, { replaceState: true });
      } else {
        await invalidateAll();
      }
    } catch (error) {
      logger.error("Failed to update tag", {
        error: error instanceof Error ? error.message : String(error),
      });
      alert("Failed to update tag");
    } finally {
      saving = false;
    }
  }

  function initiateDelete() {
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
    deleteConfirmReady = false;
  }

  async function confirmDelete() {
    if (!deleteConfirmReady) return;

    try {
      await deleteAdminTag(data.tag.id);
      await goto("/admin/tags");
    } catch (error) {
      logger.error("Failed to delete tag", {
        error: error instanceof Error ? error.message : String(error),
      });
      alert("Failed to delete tag");
    }
  }

  // Base classes for tag chip styling (matches TagChip component)
  const tagBaseClasses =
    "inline-flex items-center gap-1.25 rounded-r-sm rounded-l-xs bg-zinc-200/80 dark:bg-zinc-700/50 px-2 sm:px-1.5 py-1 sm:py-0.75 text-sm sm:text-xs text-zinc-700 dark:text-zinc-300 border-l-3 shadow-sm";
</script>

<svelte:head>
  <title>Edit {data.tag.name} | Tags | Admin</title>
</svelte:head>

<IconSprite icons={data.icons} />

<div class="space-y-6 max-w-3xl">
  <!-- Back Link -->
  <a
    href="/admin/tags"
    class="inline-flex items-center gap-1.5 text-sm text-admin-text-muted hover:text-admin-text transition-colors"
  >
    <IconArrowLeft class="w-4 h-4" />
    Back to Tags
  </a>

  <!-- Header -->
  <div>
    <h1 class="text-xl font-semibold text-admin-text">Edit Tag</h1>
    <p class="mt-1 text-sm text-admin-text-muted">
      Modify tag details and view associated projects
    </p>
  </div>

  <!-- Edit Form -->
  <div
    class="rounded-xl border border-admin-border bg-admin-surface p-6 shadow-sm shadow-black/10 dark:shadow-black/20"
  >
    <div class="grid gap-4 md:grid-cols-2">
      <Input
        label="Name"
        type="text"
        bind:value={name}
        placeholder="TypeScript"
        required
      />
      <Input
        label="Slug"
        type="text"
        bind:value={slug}
        placeholder="Leave empty to keep current"
      />
    </div>

    <div class="mt-4">
      <IconPicker bind:selectedIcon={icon} label="Icon" />
    </div>

    <div class="mt-4">
      <ColorPicker bind:selectedColor={color} />
    </div>

    <!-- Preview - rendered inline with dynamic icon SVG -->
    <div class="mt-6 pt-4 border-t border-admin-border">
      <span class="block text-sm font-medium text-admin-text mb-2">
        Preview
      </span>
      <span
        class={tagBaseClasses}
        style="border-left-color: #{color || '06b6d4'}"
      >
        {#if previewIconSvg}
          <span class="size-4.25 sm:size-3.75 [&>svg]:w-full [&>svg]:h-full">
            <!-- eslint-disable-next-line svelte/no-at-html-tags -->
            {@html previewIconSvg}
          </span>
        {/if}
        <span>{name || "Tag Name"}</span>
      </span>
    </div>

    <!-- Actions -->
    <div class="mt-6 pt-4 border-t border-admin-border flex justify-between">
      <Button variant="danger" onclick={initiateDelete}>Delete Tag</Button>
      <div class="flex gap-2">
        <Button variant="secondary" href="/admin/tags">Cancel</Button>
        <Button
          variant="primary"
          onclick={handleSave}
          disabled={saving || !name.trim()}
        >
          {saving ? "Saving..." : "Save Changes"}
        </Button>
      </div>
    </div>
  </div>

  <!-- Projects using this tag -->
  {#if data.projects.length > 0}
    <div
      class="rounded-xl border border-admin-border bg-admin-surface p-6 shadow-sm shadow-black/10 dark:shadow-black/20"
    >
      <h2 class="text-base font-medium text-admin-text mb-4">
        Projects using this tag ({data.projects.length})
      </h2>
      <ul class="space-y-2">
        {#each data.projects as project (project.id)}
          <li>
            <a
              href={`/admin/projects/${project.id}`}
              class="flex items-center justify-between p-2 -mx-2 rounded-lg hover:bg-admin-surface-hover transition-colors group"
            >
              <span class="text-admin-text group-hover:text-admin-primary">
                {project.name}
              </span>
              <IconExternalLink
                class="w-4 h-4 text-admin-text-muted opacity-0 group-hover:opacity-100 transition-opacity"
              />
            </a>
          </li>
        {/each}
      </ul>
    </div>
  {/if}

  <!-- Related Tags -->
  {#if data.relatedTags.length > 0}
    <div
      class="rounded-xl border border-admin-border bg-admin-surface p-6 shadow-sm shadow-black/10 dark:shadow-black/20"
    >
      <h2 class="text-base font-medium text-admin-text mb-4">Related Tags</h2>
      <p class="text-sm text-admin-text-muted mb-4">
        Tags that frequently appear alongside this one
      </p>
      <div class="flex flex-wrap gap-2">
        {#each data.relatedTags as tag (tag.id)}
          <TagChip
            name={tag.name}
            color={tag.color}
            icon={tag.icon}
            href={`/admin/tags/${tag.slug}`}
          />
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
  <div
    class="rounded-md bg-admin-surface-hover/50 border border-admin-border p-3"
  >
    <p class="font-medium text-admin-text">{data.tag.name}</p>
    <p class="text-sm text-admin-text-secondary">
      Used in {data.projects.length} project{data.projects.length === 1
        ? ""
        : "s"}
    </p>
  </div>
</Modal>
