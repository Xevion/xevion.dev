<script lang="ts">
  import Button from "$lib/components/admin/Button.svelte";
  import Input from "$lib/components/admin/Input.svelte";
  import Modal from "$lib/components/admin/Modal.svelte";
  import ColorPicker from "$lib/components/admin/ColorPicker.svelte";
  import IconPicker from "$lib/components/admin/IconPicker.svelte";
  import TagChip from "$lib/components/TagChip.svelte";
  import { createAdminTag, deleteAdminTag } from "$lib/api";
  import type { ApiTagWithCount } from "$lib/bindings";
  import type { CreateTagData } from "$lib/admin-types";
  import type { PageData } from "./$types";
  import IconPlus from "~icons/lucide/plus";
  import IconX from "~icons/lucide/x";
  import IconInfo from "~icons/lucide/info";
  import { invalidateAll } from "$app/navigation";
  import { getLogger } from "@logtape/logtape";
  import { css, cx } from "styled-system/css";
  import { hstack, flex, wrap, grid } from "styled-system/patterns";
  import {
    pageTitleClass,
    pageDescriptionClass,
    iconSm,
    adminCardClass,
    sectionHeadingClass,
  } from "$lib/styles/admin";

  const logger = getLogger(["admin", "tags"]);

  let { data }: { data: PageData } = $props();

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
  let deleteTarget = $state<ApiTagWithCount | null>(null);
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

  function handleTagClick(tag: ApiTagWithCount, event: MouseEvent) {
    if (deleteMode) {
      event.preventDefault();
      event.stopPropagation();
      initiateDelete(tag);
    }
    // Otherwise, let the link navigate normally
  }

  function handleTagKeyDown(tag: ApiTagWithCount, event: KeyboardEvent) {
    if (deleteMode && (event.key === "Enter" || event.key === " ")) {
      event.preventDefault();
      initiateDelete(tag);
    }
  }

  function initiateDelete(tag: ApiTagWithCount) {
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

<div class={css({ spaceY: "6" })}>
  <!-- Header -->
  <div class={hstack({ justify: "space-between", gap: "0" })}>
    <div>
      <div class={hstack({ gap: "2" })}>
        <h1 class={pageTitleClass}>Tags</h1>
        <span
          class={css({
            color: "admin.textMuted",
            cursor: "help",
            transition: "colors",
            _hover: { color: "admin.text" },
          })}
          title="Hold Shift and click a tag to delete it"
        >
          <IconInfo class={iconSm} />
        </span>
      </div>
      <p class={pageDescriptionClass}>Manage project tags and categories</p>
    </div>
    <Button
      variant="primary"
      onclick={() => (showCreateForm = !showCreateForm)}
    >
      {#if showCreateForm}
        <IconX class={cx(iconSm, css({ mr: "2" }))} />
      {:else}
        <IconPlus class={cx(iconSm, css({ mr: "2" }))} />
      {/if}
      {showCreateForm ? "Cancel" : "New Tag"}
    </Button>
  </div>

  <!-- Create Form -->
  {#if showCreateForm}
    <div class={adminCardClass}>
      <h3 class={sectionHeadingClass}>Create New Tag</h3>
      <div class={grid({ columns: { md: 2 }, gap: "4" })}>
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
      <div class={css({ mt: "4" })}>
        <IconPicker bind:selectedIcon={createIcon} label="Icon (optional)" />
      </div>
      <div class={css({ mt: "4" })}>
        <ColorPicker bind:selectedColor={createColor} />
      </div>
      <div class={flex({ justify: "flex-end", gap: "2", mt: "4" })}>
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
    <div class={css({ textAlign: "center", py: "12" })}>
      <p class={css({ color: "admin.textMuted", mb: "4" })}>No tags yet</p>
      <Button variant="primary" onclick={() => (showCreateForm = true)}>
        Create your first tag
      </Button>
    </div>
  {:else}
    <div class={css({ spaceY: "3" })}>
      <!-- Delete mode indicator -->
      <div
        class={cx(
          flex({
            align: "center",
            h: "6",
            transition: "all",
            transitionDuration: "150ms",
          }),
          deleteMode ? css({ opacity: "1" }) : css({ opacity: "0" }),
        )}
      >
        <span
          class={hstack({
            gap: "1.5",
            fontSize: "sm",
            color: "red.500",
            fontWeight: "medium",
            _dark: { color: "red.400" },
          })}
        >
          <IconX class={iconSm} />
          Click a tag to delete it
        </span>
      </div>

      <!-- Tags -->
      <div class={wrap({ gap: "2", maxW: "48rem" })}>
        {#each data.tags as tag (tag.id)}
          <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
          <div
            onclick={(e) => handleTagClick(tag, e)}
            onkeydown={(e) => handleTagKeyDown(tag, e)}
            role={deleteMode ? "button" : undefined}
            tabindex={deleteMode ? 0 : undefined}
            class={css({ display: "contents" })}
          >
            <TagChip
              name={tag.name}
              color={deleteMode ? "ef4444" : tag.color}
              icon={tag.icon}
              href={`/admin/tags/${tag.slug}`}
              class={cx(
                css({ transition: "all", transitionDuration: "150ms" }),
                deleteMode
                  ? css({
                      bg: "red.100/80",
                      cursor: "pointer",
                      _dark: { bg: "red.900/40" },
                    })
                  : "",
              )}
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
      class={css({
        rounded: "md",
        bg: "admin.surfaceHover/50",
        borderWidth: "1px",
        borderColor: "admin.border",
        p: "3",
      })}
    >
      <p class={css({ fontWeight: "medium", color: "admin.text" })}>
        {deleteTarget.name}
      </p>
      <p class={css({ fontSize: "sm", color: "admin.textSecondary" })}>
        Used in {deleteTarget.projectCount} project{deleteTarget.projectCount ===
        1
          ? ""
          : "s"}
      </p>
    </div>
  {/if}
</Modal>
