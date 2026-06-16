<script lang="ts">
  import Button from "$lib/components/admin/Button.svelte";
  import Input from "$lib/components/admin/Input.svelte";
  import Modal from "$lib/components/admin/Modal.svelte";
  import ColorPicker from "$lib/components/admin/ColorPicker.svelte";
  import IconPicker from "$lib/components/admin/IconPicker.svelte";
  import TagChip from "$lib/components/TagChip.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import { updateAdminTag, deleteAdminTag } from "$lib/api";
  import { goto, invalidateAll } from "$app/navigation";
  import type { PageData } from "./$types";
  import IconArrowLeft from "~icons/lucide/arrow-left";
  import IconExternalLink from "~icons/lucide/external-link";
  import { getLogger } from "@logtape/logtape";
  import { toast } from "$lib/toast";
  import { css, cx } from "styled-system/css";
  import { flex, hstack, wrap, grid } from "styled-system/patterns";
  import {
    pageTitleClass,
    pageDescriptionClass,
    adminCardClass,
    sectionHeadingClass,
    iconSm,
  } from "$lib/styles/admin";

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
  let fieldErrors = $state<Record<string, string>>({});

  // Delete state
  let deleteModalOpen = $state(false);
  let deleteConfirmReady = $state(false);
  let deleteTimeout: ReturnType<typeof setTimeout> | null = null;

  async function handleSave() {
    if (!name.trim()) return;

    saving = true;
    fieldErrors = {};
    const result = await updateAdminTag({
      id: data.tag.id,
      name: name.trim(),
      slug: slug.trim() || undefined,
      icon: icon || undefined,
      color: color,
    });
    if (result.isErr) {
      logger.error("Failed to update tag", { error: result.error });
      fieldErrors = result.error.fieldErrors ?? {};
      toast.error(result.error.message);
      saving = false;
      return;
    }

    // If slug changed, navigate to new URL
    const newSlug = slug.trim() || data.tag.slug;
    if (newSlug !== data.tag.slug) {
      await goto(`/admin/tags/${newSlug}`, { replaceState: true });
    } else {
      await invalidateAll();
    }
    saving = false;
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

    const result = await deleteAdminTag(data.tag.id);
    if (result.isErr) {
      logger.error("Failed to delete tag", { error: result.error });
      toast.error(result.error.message);
      return;
    }
    await goto("/admin/tags");
  }

  // Base classes for tag chip styling (matches TagChip component)
  const tagBaseClasses = css({
    display: "inline-flex",
    alignItems: "center",
    gap: "5px",
    borderTopRightRadius: "sm",
    borderBottomRightRadius: "sm",
    borderTopLeftRadius: "xs",
    borderBottomLeftRadius: "xs",
    bg: "zinc.200/80",
    px: "2",
    py: "1",
    fontSize: "sm",
    color: "zinc.700",
    borderLeftWidth: "3px",
    shadow: "sm",
    _dark: { bg: "zinc.700/50", color: "zinc.300" },
    sm: { px: "1.5", py: "3px", fontSize: "xs" },
  });
</script>

<svelte:head>
  <title>Edit {data.tag.name} | Tags | Admin</title>
</svelte:head>

<div class={css({ spaceY: "6", maxW: "48rem" })}>
  <!-- Back Link -->
  <a
    href="/admin/tags"
    class={css({
      display: "inline-flex",
      alignItems: "center",
      gap: "1.5",
      fontSize: "sm",
      color: "admin.textMuted",
      _hover: { color: "admin.text" },
      transition: "colors",
    })}
  >
    <IconArrowLeft class={iconSm} />
    Back to Tags
  </a>

  <!-- Header -->
  <div>
    <h1 class={pageTitleClass}>Edit Tag</h1>
    <p class={pageDescriptionClass}>
      Modify tag details and view associated projects
    </p>
  </div>

  <!-- Edit Form -->
  <div class={adminCardClass}>
    <div class={grid({ columns: { md: 2 }, gap: "4" })}>
      <Input
        label="Name"
        type="text"
        bind:value={name}
        placeholder="TypeScript"
        required
        error={fieldErrors.name}
      />
      <Input
        label="Slug"
        type="text"
        bind:value={slug}
        placeholder="Leave empty to keep current"
        error={fieldErrors.slug}
        inputClass={css({ fontFamily: "geist" })}
      />
    </div>

    <div class={css({ mt: "4" })}>
      <IconPicker bind:selectedIcon={icon} label="Icon" />
    </div>

    <div class={css({ mt: "4" })}>
      <ColorPicker bind:selectedColor={color} />
    </div>

    <!-- Preview -->
    <div
      class={css({
        mt: "6",
        pt: "4",
        borderTopWidth: "1px",
        borderColor: "admin.border",
      })}
    >
      <span
        class={css({
          display: "block",
          fontSize: "sm",
          fontWeight: "medium",
          color: "admin.text",
          mb: "2",
        })}>Preview</span
      >
      <span
        class={tagBaseClasses}
        style="border-left-color: #{color || '06b6d4'}"
      >
        {#if icon}
          <Icon
            {icon}
            sizeClass={css({ w: "4", h: "4", sm: { w: "3.5", h: "3.5" } })}
          /><!-- Icon has responsive sizing, can't use iconSm -->
        {/if}
        <span>{name || "Tag Name"}</span>
      </span>
    </div>

    <!-- Actions -->
    <div
      class={flex({
        justify: "space-between",
        mt: "6",
        pt: "4",
        borderTopWidth: "1px",
        borderColor: "admin.border",
      })}
    >
      <Button variant="danger" onclick={initiateDelete}>Delete Tag</Button>
      <div class={flex({ gap: "2" })}>
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
    <div class={adminCardClass}>
      <h2 class={sectionHeadingClass}>
        Projects using this tag ({data.projects.length})
      </h2>
      <ul class={css({ spaceY: "2" })}>
        {#each data.projects as project (project.id)}
          <li>
            <a
              href={`/admin/projects/${project.id}`}
              class={hstack({
                justify: "space-between",
                gap: "0",
                p: "2",
                mx: "-2",
                rounded: "lg",
                transition: "colors",
                _hover: { bg: "admin.surfaceHover" },
              })}
            >
              <span
                class={css({
                  color: "admin.text",
                  _groupHover: { color: "admin.accent" },
                })}
              >
                {project.name}
              </span>
              <IconExternalLink
                class={cx(
                  iconSm,
                  css({
                    color: "admin.textMuted",
                    opacity: "0",
                    transition: "opacity",
                  }),
                )}
              />
            </a>
          </li>
        {/each}
      </ul>
    </div>
  {/if}

  <!-- Related Tags -->
  {#if data.relatedTags.length > 0}
    <div class={adminCardClass}>
      <h2 class={sectionHeadingClass}>Related Tags</h2>
      <p class={css({ fontSize: "sm", color: "admin.textMuted", mb: "4" })}>
        Tags that frequently appear alongside this one
      </p>
      <div class={wrap({ gap: "2" })}>
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
    class={css({
      rounded: "md",
      bg: "admin.surfaceHover/50",
      borderWidth: "1px",
      borderColor: "admin.border",
      p: "3",
    })}
  >
    <p class={css({ fontWeight: "medium", color: "admin.text" })}>
      {data.tag.name}
    </p>
    <p class={css({ fontSize: "sm", color: "admin.textSecondary" })}>
      Used in {data.projects.length} project{data.projects.length === 1
        ? ""
        : "s"}
    </p>
  </div>
</Modal>
