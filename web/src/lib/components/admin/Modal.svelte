<script lang="ts">
  import { cn } from "$lib/utils";
  import Button from "./Button.svelte";

  interface Props {
    open: boolean;
    title?: string;
    description?: string;
    confirmText?: string;
    cancelText?: string;
    confirmVariant?: "primary" | "danger";
    onconfirm?: () => void;
    oncancel?: () => void;
    children?: import("svelte").Snippet;
    class?: string;
  }

  let {
    open = $bindable(false),
    title = "Confirm",
    description,
    confirmText = "Confirm",
    cancelText = "Cancel",
    confirmVariant = "primary",
    onconfirm,
    oncancel,
    children,
    class: className,
  }: Props = $props();

  function handleCancel() {
    open = false;
    oncancel?.();
  }

  function handleConfirm() {
    open = false;
    onconfirm?.();
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      handleCancel();
    }
  }
</script>

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm p-4"
    onclick={handleBackdropClick}
    onkeydown={(e) => e.key === "Escape" && handleCancel()}
    role="presentation"
    tabindex="-1"
  >
    <div
      class={cn(
        "relative w-full max-w-md rounded-xl bg-admin-surface border border-admin-border p-8 shadow-xl shadow-black/20 dark:shadow-black/50",
        className,
      )}
      role="dialog"
      aria-modal="true"
    >
      {#if title}
        <h2 class="text-lg font-semibold text-admin-text mb-2">
          {title}
        </h2>
      {/if}

      {#if description}
        <p class="text-sm text-admin-text-secondary mb-4">
          {description}
        </p>
      {/if}

      {#if children}
        <div class="mb-4">
          {@render children()}
        </div>
      {/if}

      <div class="flex justify-end gap-3">
        <Button variant="secondary" onclick={handleCancel}>
          {cancelText}
        </Button>
        <Button variant={confirmVariant} onclick={handleConfirm}>
          {confirmText}
        </Button>
      </div>
    </div>
  </div>
{/if}
