<script lang="ts">
  import { cn } from "$lib/utils";
  import { dndzone } from "svelte-dnd-action";
  import { flip } from "svelte/animate";
  import type { ProjectMedia } from "$lib/admin-types";
  import {
    uploadProjectMedia,
    deleteProjectMedia,
    reorderProjectMedia,
  } from "$lib/api";
  import MediaItem from "./MediaItem.svelte";
  import Modal from "./Modal.svelte";
  import { getLogger } from "@logtape/logtape";
  import IconCloudUpload from "~icons/lucide/cloud-upload";
  import IconAlertCircle from "~icons/lucide/alert-circle";
  import IconLoader from "~icons/lucide/loader-2";
  import IconX from "~icons/lucide/x";

  const logger = getLogger(["admin", "components", "MediaManager"]);

  interface Props {
    projectId: string | null;
    media?: ProjectMedia[];
    onchange?: (media: ProjectMedia[]) => void;
    class?: string;
  }

  let { projectId, media = [], onchange, class: className }: Props = $props();

  // Local media state (for reordering) - needs to be mutable for drag-drop
  // eslint-disable-next-line svelte/prefer-writable-derived -- intentional: svelte-dnd-action requires mutable array
  let mediaItems = $state<ProjectMedia[]>([]);

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
  let deletingMedia = $state<ProjectMedia | null>(null);

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
  function handleDndConsider(e: CustomEvent<{ items: ProjectMedia[] }>) {
    mediaItems = e.detail.items;
  }

  async function handleDndFinalize(e: CustomEvent<{ items: ProjectMedia[] }>) {
    mediaItems = e.detail.items;
    onchange?.(mediaItems);

    // Call reorder API
    if (projectId) {
      try {
        await reorderProjectMedia(
          projectId,
          mediaItems.map((m) => m.id),
        );
        logger.info("Media reordered", { projectId, count: mediaItems.length });
      } catch (err) {
        logger.error("Failed to reorder media", { error: err });
        showError("Failed to save new order");
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

    try {
      const media = await uploadProjectMedia(projectId, file, (progress) => {
        uploadQueue = uploadQueue.map((t) =>
          t.id === taskId ? { ...t, progress } : t,
        );
      });

      // Add to media items
      mediaItems = [...mediaItems, media];
      onchange?.(mediaItems);

      // Remove from queue
      uploadQueue = uploadQueue.filter((t) => t.id !== taskId);

      logger.info("Media uploaded", { projectId, mediaId: media.id });
    } catch (err) {
      logger.error("Upload failed", { error: err, filename: file.name });
      uploadQueue = uploadQueue.map((t) =>
        t.id === taskId ? { ...t, status: "error", error: String(err) } : t,
      );
    }
  }

  function removeUploadTask(taskId: string) {
    uploadQueue = uploadQueue.filter((t) => t.id !== taskId);
  }

  // Delete handlers
  function handleDeleteClick(media: ProjectMedia) {
    deletingMedia = media;
    deleteModalOpen = true;
  }

  async function confirmDelete() {
    if (!projectId || !deletingMedia) return;

    try {
      await deleteProjectMedia(projectId, deletingMedia.id);
      mediaItems = mediaItems.filter((m) => m.id !== deletingMedia!.id);
      onchange?.(mediaItems);
      logger.info("Media deleted", { projectId, mediaId: deletingMedia.id });
    } catch (err) {
      logger.error("Failed to delete media", { error: err });
      showError("Failed to delete media");
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

<div class={cn("space-y-1.5", className)}>
  <div class="block text-sm font-medium text-admin-text">Media</div>

  {#if !projectId}
    <!-- Disabled state for new projects -->
    <div
      class="rounded-lg border-2 border-dashed border-admin-border bg-admin-bg-secondary p-8 text-center"
    >
      <IconCloudUpload class="size-8 text-admin-text-muted mb-2 mx-auto" />
      <p class="text-sm text-admin-text-muted">
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
        class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3 mb-3"
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
      class={cn(
        "rounded-lg border-2 border-dashed p-6 text-center cursor-pointer transition-colors",
        isDraggingFile
          ? "border-admin-accent bg-admin-accent/10"
          : "border-admin-border bg-admin-bg-secondary hover:border-admin-text-muted hover:bg-admin-surface",
      )}
      ondragenter={handleDragEnter}
      ondragleave={handleDragLeave}
      ondragover={handleDragOver}
      ondrop={handleDrop}
      onclick={() => fileInputRef?.click()}
      onkeydown={(e) => e.key === "Enter" && fileInputRef?.click()}
    >
      <IconCloudUpload
        class={cn(
          "size-6 mb-2 mx-auto",
          isDraggingFile ? "text-admin-accent" : "text-admin-text-muted",
        )}
      />
      <p class="text-sm text-admin-text">
        {isDraggingFile
          ? "Drop files here"
          : "Drop files here or click to upload"}
      </p>
      <p class="text-xs text-admin-text-muted mt-1">
        JPEG, PNG, GIF, WebP, MP4, WebM
      </p>
    </div>

    <input
      bind:this={fileInputRef}
      type="file"
      accept={SUPPORTED_TYPES.join(",")}
      multiple
      class="hidden"
      onchange={handleFileInputChange}
    />

    <!-- Upload queue -->
    {#if uploadQueue.length > 0}
      <div class="space-y-2 mt-3">
        {#each uploadQueue as task (task.id)}
          <div
            class="flex items-center gap-3 p-2 rounded-lg bg-admin-bg-secondary border border-admin-border"
          >
            {#if task.status === "error"}
              <IconAlertCircle class="size-4 text-red-500 shrink-0" />
            {:else}
              <IconLoader
                class="size-4 text-admin-text-muted animate-spin shrink-0"
              />
            {/if}
            <div class="flex-1 min-w-0">
              <p class="text-sm text-admin-text truncate">{task.file.name}</p>
              {#if task.status === "uploading"}
                <div
                  class="h-1.5 bg-admin-border rounded-full mt-1 overflow-hidden"
                >
                  <div
                    class="h-full bg-admin-accent transition-all duration-200"
                    style="width: {task.progress}%"
                  ></div>
                </div>
              {:else if task.status === "error"}
                <p class="text-xs text-red-500 truncate">{task.error}</p>
              {/if}
            </div>
            {#if task.status === "error"}
              <button
                type="button"
                onclick={() => removeUploadTask(task.id)}
                class="text-admin-text-muted hover:text-admin-text p-1"
                aria-label="Dismiss error"
              >
                <IconX class="size-4" />
              </button>
            {:else}
              <span class="text-xs text-admin-text-muted">{task.progress}%</span
              >
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    <!-- Error message -->
    {#if errorMessage}
      <div
        class="flex items-center gap-2 p-3 mt-2 rounded-lg bg-red-500/10 border border-red-500/30 text-red-500 text-sm"
      >
        <IconAlertCircle class="size-4 shrink-0" />
        <span>{errorMessage}</span>
        <button
          type="button"
          onclick={clearError}
          class="ml-auto hover:text-red-400"
          aria-label="Dismiss error"
        >
          <IconX class="size-4" />
        </button>
      </div>
    {/if}
  {/if}

  <p class="text-xs text-admin-text-muted">
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
