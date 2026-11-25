<script lang="ts">
  import { css, cx } from "styled-system/css";
  import { center, flex } from "styled-system/patterns";
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
    class={center({
      position: "fixed",
      inset: "0",
      zIndex: 50,
      bg: "black/60",
      backdropFilter: "blur(4px)",
      p: "4",
    })}
    onclick={handleBackdropClick}
    onkeydown={(e) => e.key === "Escape" && handleCancel()}
    role="presentation"
    tabindex="-1"
  >
    <div
      class={cx(
        css({
          position: "relative",
          w: "full",
          maxW: "md",
          rounded: "xl",
          bg: "admin.surface",
          borderWidth: "1px",
          borderColor: "admin.border",
          p: "8",
          shadow: "xl",
          _dark: { shadowColor: "black/50" },
        }),
        className,
      )}
      role="dialog"
      aria-modal="true"
    >
      {#if title}
        <h2
          class={css({
            fontSize: "lg",
            fontWeight: "semibold",
            color: "admin.text",
            mb: "2",
          })}
        >
          {title}
        </h2>
      {/if}

      {#if description}
        <p
          class={css({ fontSize: "sm", color: "admin.textSecondary", mb: "4" })}
        >
          {description}
        </p>
      {/if}

      {#if children}
        <div class={css({ mb: "4" })}>
          {@render children()}
        </div>
      {/if}

      <div class={flex({ justify: "flex-end", gap: "3" })}>
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
