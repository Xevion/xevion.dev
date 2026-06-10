<script lang="ts">
  import { css, cx } from "styled-system/css";
  import { hstack, grid } from "styled-system/patterns";
  import { dndzone } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import type { ApiProjectMedia } from "$lib/bindings";
  import {
    uploadProjectMedia,
    deleteProjectMedia,
    reorderProjectMedia,
  } from "$lib/api";
  import MediaItem from "./MediaItem.svelte";
  import Modal from "./Modal.svelte";
  import { getLogger } from "@logtape/logtape";
  import {
    labelClass,
    helpTextClass,
    fieldWrapperClass,
    iconSm,
  } from "$lib/styles/admin";
  import IconCloudUpload from "~icons/lucide/cloud-upload";
  import IconAlertCircle from "~icons/lucide/alert-circle";
  import IconLoader from "~icons/lucide/loader-2";
  import IconX from "~icons/lucide/x";

  const logger = getLogger(["admin", "components", "MediaManager"]);

  interface Props {
    projectId: string | null;
    media?: ApiProjectMedia[];
    onchange?: (media: ApiProjectMedia[]) => void;
    class?: string;
  }

  let { projectId, media = [], onchange, class: className }: Props = $props();

  // Local media state (for reordering) - needs to be mutable for drag-drop
  // eslint-disable-next-line svelte/prefer-writable-derived -- intentional: svelte-dnd-action requires mutable array
  let mediaItems = $state<ApiProjectMedia[]>([]);

  // Sync from props when they change
  $effect(() => {
    mediaItems = [...media];
  });

  // Upload state
  interface UploadTask {
    id: string;
    file: File;
    progress: number;
    status: "uploading" | "done" | "error";
    error?: string;
  }
  let uploadQueue = $state<UploadTask[]>([]);

  // UI state
  let isDraggingFile = $state(false);
  let errorMessage = $state<string | null>(null);
  let fileInputRef: HTMLInputElement | null = $state(null);

  // Delete confirmation
  let deleteModalOpen = $state(false);
  let deletingMedia = $state<ApiProjectMedia | null>(null);

  const flipDurationMs = 150;
  const SUPPORTED_IMAGE_TYPES = [
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
    "image/avif",
  ];
  const SUPPORTED_VIDEO_TYPES = ["video/mp4", "video/webm", "video/quicktime"];
  const SUPPORTED_TYPES = [...SUPPORTED_IMAGE_TYPES, ...SUPPORTED_VIDEO_TYPES];

  // Drag and drop reorder handlers
  function handleDndConsider(e: CustomEvent<{ items: ApiProjectMedia[] }>) {
    mediaItems = e.detail.items;
  }

  async function handleDndFinalize(
    e: CustomEvent<{ items: ApiProjectMedia[] }>,
  ) {
    mediaItems = e.detail.items;
    onchange?.(mediaItems);

    // Call reorder API
    if (projectId) {
      const result = await reorderProjectMedia(
        projectId,
        mediaItems.map((m) => m.id),
      );
      if (result.isErr) {
        logger.error("Failed to reorder media", { error: result.error });
        showError(result.error.message);
      } else {
        logger.info("Media reordered", { projectId, count: mediaItems.length });
      }
    }
  }

  // File upload handlers
  function handleDragEnter(e: DragEvent) {
    e.preventDefault();
    isDraggingFile = true;
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    // Only set false if leaving the drop zone entirely
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const x = e.clientX;
    const y = e.clientY;
    if (x < rect.left || x > rect.right || y < rect.top || y > rect.bottom) {
      isDraggingFile = false;
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDraggingFile = true;
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDraggingFile = false;

    const files = e.dataTransfer?.files;
    if (files && files.length > 0) {
      handleFiles(Array.from(files));
    }
  }

  function handleFileInputChange(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files && input.files.length > 0) {
      handleFiles(Array.from(input.files));
      input.value = ""; // Reset for re-upload of same file
    }
  }

  function handleFiles(files: File[]) {
    clearError();

    const validFiles: File[] = [];
    const invalidTypes: string[] = [];

    for (const file of files) {
      if (SUPPORTED_TYPES.includes(file.type)) {
        validFiles.push(file);
      } else {
        invalidTypes.push(file.name);
      }
    }

    if (invalidTypes.length > 0) {
      showError(
        `Unsupported file type${invalidTypes.length > 1 ? "s" : ""}: ${invalidTypes.join(", ")}`,
      );
    }

    for (const file of validFiles) {
      uploadFile(file);
    }
  }

  async function uploadFile(file: File) {
    if (!projectId) return;

    const taskId = crypto.randomUUID();
    const task: UploadTask = {
      id: taskId,
      file,
      progress: 0,
      status: "uploading",
    };

    uploadQueue = [...uploadQueue, task];

    const result = await uploadProjectMedia(projectId, file, (progress) => {
      uploadQueue = uploadQueue.map((t) =>
        t.id === taskId ? { ...t, progress } : t,
      );
    });

    if (result.isErr) {
      logger.error("Upload failed", {
        error: result.error,
        filename: file.name,
      });
      uploadQueue = uploadQueue.map((t) =>
        t.id === taskId
          ? { ...t, status: "error", error: result.error.message }
          : t,
      );
      return;
    }

    const media = result.value;

    // Add to media items
    mediaItems = [...mediaItems, media];
    onchange?.(mediaItems);

    // Remove from queue
    uploadQueue = uploadQueue.filter((t) => t.id !== taskId);

    logger.info("Media uploaded", { projectId, mediaId: media.id });
  }

  function removeUploadTask(taskId: string) {
    uploadQueue = uploadQueue.filter((t) => t.id !== taskId);
  }

  // Delete handlers
  function handleDeleteClick(media: ApiProjectMedia) {
    deletingMedia = media;
    deleteModalOpen = true;
  }

  async function confirmDelete() {
    if (!projectId || !deletingMedia) return;

    const result = await deleteProjectMedia(projectId, deletingMedia.id);
    if (result.isErr) {
      logger.error("Failed to delete media", { error: result.error });
      showError(result.error.message);
    } else {
      mediaItems = mediaItems.filter((m) => m.id !== deletingMedia!.id);
      onchange?.(mediaItems);
      logger.info("Media deleted", { projectId, mediaId: deletingMedia.id });
    }

    deletingMedia = null;
  }

  // Error handling
  function showError(msg: string) {
    errorMessage = msg;
    setTimeout(() => {
      if (errorMessage === msg) {
        errorMessage = null;
      }
    }, 5000);
  }

  function clearError() {
    errorMessage = null;
  }
</script>

<div class={cx(fieldWrapperClass, className)}>
  <div class={labelClass}>Media</div>

  {#if !projectId}
    <!-- Disabled state for new projects -->
    <div
      class={css({
        rounded: "lg",
        borderWidth: "2px",
        borderStyle: "dashed",
        borderColor: "admin.border",
        bg: "admin.bgSecondary",
        p: "8",
        textAlign: "center",
      })}
    >
      <IconCloudUpload
        class={css({
          w: "8",
          h: "8",
          color: "admin.textMuted",
          mb: "2",
          mx: "auto",
        })}
      />
      <p class={css({ fontSize: "sm", color: "admin.textMuted" })}>
        Save the project first to enable media uploads
      </p>
    </div>
  {:else}
    <!-- Media grid (if has media) -->
    {#if mediaItems.length > 0}
      <div
        use:dndzone={{
          items: mediaItems,
          flipDurationMs,
          dropTargetStyle: {},
        }}
        onconsider={handleDndConsider}
        onfinalize={handleDndFinalize}
        class={grid({
          columns: { base: 2, sm: 3, md: 4 },
          gap: "3",
          mb: "3",
        })}
      >
        {#each mediaItems as item (item.id)}
          <div animate:flip={{ duration: flipDurationMs }}>
            <MediaItem media={item} ondelete={() => handleDeleteClick(item)} />
          </div>
        {/each}
      </div>
    {/if}

    <!-- Upload drop zone -->
    <div
      role="button"
      tabindex="0"
      class={cx(
        css({
          rounded: "lg",
          borderWidth: "2px",
          borderStyle: "dashed",
          p: "6",
          textAlign: "center",
          cursor: "pointer",
          transition: "colors",
        }),
        isDraggingFile
          ? css({ borderColor: "admin.accent", bg: "admin.accent/10" })
          : css({
              borderColor: "admin.border",
              bg: "admin.bgSecondary",
              _hover: { borderColor: "admin.textMuted", bg: "admin.surface" },
            }),
      )}
      ondragenter={handleDragEnter}
      ondragleave={handleDragLeave}
      ondragover={handleDragOver}
      ondrop={handleDrop}
      onclick={() => fileInputRef?.click()}
      onkeydown={(e) => e.key === "Enter" && fileInputRef?.click()}
    >
      <IconCloudUpload
        class={cx(
          css({ w: "6", h: "6", mb: "2", mx: "auto" }),
          isDraggingFile
            ? css({ color: "admin.accent" })
            : css({ color: "admin.textMuted" }),
        )}
      />
      <p class={css({ fontSize: "sm", color: "admin.text" })}>
        {isDraggingFile
          ? "Drop files here"
          : "Drop files here or click to upload"}
      </p>
      <p class={css({ fontSize: "xs", color: "admin.textMuted", mt: "1" })}>
        JPEG, PNG, GIF, WebP, MP4, WebM
      </p>
    </div>

    <input
      bind:this={fileInputRef}
      type="file"
      accept={SUPPORTED_TYPES.join(",")}
      multiple
      class={css({ display: "none" })}
      onchange={handleFileInputChange}
    />

    <!-- Upload queue -->
    {#if uploadQueue.length > 0}
      <div class={css({ spaceY: "2", mt: "3" })}>
        {#each uploadQueue as task (task.id)}
          <div
            class={hstack({
              gap: "3",
              p: "2",
              rounded: "lg",
              bg: "admin.bgSecondary",
              borderWidth: "1px",
              borderColor: "admin.border",
            })}
          >
            {#if task.status === "error"}
              <IconAlertCircle
                class={cx(iconSm, css({ color: "red.500", flexShrink: "0" }))}
              />
            {:else}
              <IconLoader
                class={cx(
                  iconSm,
                  css({
                    color: "admin.textMuted",
                    animation: "spin",
                    flexShrink: "0",
                  }),
                )}
              />
            {/if}
            <div class={css({ flex: "1", minW: "0" })}>
              <p
                class={css({
                  fontSize: "sm",
                  color: "admin.text",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  whiteSpace: "nowrap",
                })}
              >
                {task.file.name}
              </p>
              {#if task.status === "uploading"}
                <div
                  class={css({
                    h: "1.5",
                    bg: "admin.border",
                    rounded: "full",
                    mt: "1",
                    overflow: "hidden",
                  })}
                >
                  <div
                    class={css({
                      h: "full",
                      bg: "admin.accent",
                      transition: "all",
                      transitionDuration: "200ms",
                    })}
                    style="width: {task.progress}%"
                  ></div>
                </div>
              {:else if task.status === "error"}
                <p
                  class={css({
                    fontSize: "xs",
                    color: "red.500",
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                    whiteSpace: "nowrap",
                  })}
                >
                  {task.error}
                </p>
              {/if}
            </div>
            {#if task.status === "error"}
              <button
                type="button"
                onclick={() => removeUploadTask(task.id)}
                class={css({
                  color: "admin.textMuted",
                  _hover: { color: "admin.text" },
                  p: "1",
                })}
                aria-label="Dismiss error"
              >
                <IconX class={iconSm} />
              </button>
            {:else}
              <span class={helpTextClass}>{task.progress}%</span>
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    <!-- Error message -->
    {#if errorMessage}
      <div
        class={hstack({
          gap: "2",
          p: "3",
          mt: "2",
          rounded: "lg",
          bg: "red.500/10",
          borderWidth: "1px",
          borderColor: "red.500/30",
          color: "red.500",
          fontSize: "sm",
        })}
      >
        <IconAlertCircle class={cx(iconSm, css({ flexShrink: "0" }))} />
        <span>{errorMessage}</span>
        <button
          type="button"
          onclick={clearError}
          class={css({ ml: "auto", _hover: { color: "red.400" } })}
          aria-label="Dismiss error"
        >
          <IconX class={iconSm} />
        </button>
      </div>
    {/if}
  {/if}

  <p class={helpTextClass}>
    {#if projectId}
      Drag to reorder. First image is shown as the project thumbnail.
    {:else}
      Media can be uploaded after saving the project.
    {/if}
  </p>
</div>

<!-- Delete confirmation modal -->
<Modal
  bind:open={deleteModalOpen}
  title="Delete Media"
  description="Are you sure you want to delete this media? This action cannot be undone."
  confirmText="Delete"
  confirmVariant="danger"
  onconfirm={confirmDelete}
  oncancel={() => (deletingMedia = null)}
/>
