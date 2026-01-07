<script lang="ts">
  import { PGP_KEY_METADATA } from "$lib/pgp/key-info";
  import { OverlayScrollbarsComponent } from "overlayscrollbars-svelte";
  import "overlayscrollbars/overlayscrollbars.css";
  import IconDownload from "~icons/material-symbols/download-rounded";
  import IconCopy from "~icons/material-symbols/content-copy-rounded";
  import { fade, scale } from "svelte/transition";

  interface Props {
    open: boolean;
  }

  let { open = $bindable(false) }: Props = $props();

  let copySuccess = $state(false);
  let keyContent = $state<string>("");
  let loading = $state(false);

  // Fetch key content when modal opens
  $effect(() => {
    if (open && !keyContent && !loading) {
      loading = true;
      fetch("/publickey.asc")
        .then((res) => res.text())
        .then((text) => {
          keyContent = text;
          loading = false;
        })
        .catch((err) => {
          console.error("Failed to fetch PGP key:", err);
          loading = false;
        });
    }
  });

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      open = false;
    }
  }

  function handleClose() {
    open = false;
  }

  async function copyToClipboard() {
    if (!keyContent) return;
    try {
      await navigator.clipboard.writeText(keyContent);
      copySuccess = true;
      setTimeout(() => {
        copySuccess = false;
      }, 2000);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  }

  function downloadKey() {
    const a = document.createElement("a");
    a.href = "/publickey.asc";
    a.download = "publickey.asc";
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
  }
</script>

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-[2px] p-4"
    onclick={handleBackdropClick}
    onkeydown={(e) => e.key === "Escape" && handleClose()}
    role="presentation"
    tabindex="-1"
    transition:fade={{ duration: 200 }}
  >
    <div
      class="relative w-full max-w-2xl rounded-lg bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 p-6 shadow-xl"
      role="dialog"
      aria-modal="true"
      aria-labelledby="pgp-modal-title"
      transition:scale={{ duration: 200, start: 0.95 }}
    >
      <div class="flex items-start justify-between mb-4">
        <h2
          id="pgp-modal-title"
          class="text-xl font-semibold text-zinc-900 dark:text-white"
        >
          PGP Public Key
        </h2>
        <button
          onclick={handleClose}
          class="text-zinc-500 hover:text-zinc-700 dark:text-zinc-400 dark:hover:text-zinc-200 transition-colors"
          aria-label="Close modal"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-6 w-6"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      </div>

      <!-- Fingerprint -->
      <div
        class="mb-4 p-3 bg-zinc-100 dark:bg-zinc-800 rounded border border-zinc-200 dark:border-zinc-700"
      >
        <div class="text-xs font-medium text-zinc-600 dark:text-zinc-400 mb-1">
          Fingerprint
        </div>
        <div
          class="font-mono text-sm text-zinc-900 dark:text-zinc-100 break-all"
        >
          {PGP_KEY_METADATA.fingerprint}
        </div>
      </div>

      <!-- Key Content -->
      <div
        class="mb-4 border border-zinc-200 dark:border-zinc-700 rounded overflow-hidden"
      >
        {#if loading}
          <div class="p-4 text-center text-zinc-600 dark:text-zinc-400">
            Loading key...
          </div>
        {:else if keyContent}
          <OverlayScrollbarsComponent
            options={{
              scrollbars: { autoHide: "leave", autoHideDelay: 800 },
            }}
            defer
            style="max-height: 400px"
          >
            <pre
              class="p-4 text-xs font-mono text-zinc-800 dark:text-zinc-200 bg-zinc-50 dark:bg-zinc-900/50 overflow-x-auto">{keyContent}</pre>
          </OverlayScrollbarsComponent>
        {:else}
          <div class="p-4 text-center text-zinc-600 dark:text-zinc-400">
            Failed to load key
          </div>
        {/if}
      </div>

      <!-- Action Buttons -->
      <div class="flex gap-3 justify-end">
        <button
          onclick={downloadKey}
          class="flex items-center gap-2 px-4 py-2 rounded-sm bg-zinc-100 dark:bg-zinc-800 text-zinc-800 dark:text-zinc-100 hover:bg-zinc-200 dark:hover:bg-zinc-700 transition-colors"
        >
          <IconDownload class="size-4" />
          <span class="text-sm font-medium">Download</span>
        </button>
        <button
          onclick={copyToClipboard}
          class="flex items-center gap-2 px-4 py-2 rounded-sm bg-zinc-900 dark:bg-zinc-100 text-white dark:text-zinc-900 hover:bg-zinc-800 dark:hover:bg-zinc-200 transition-colors"
        >
          <IconCopy class="size-4" />
          <span class="text-sm font-medium"
            >{copySuccess ? "Copied!" : "Copy to Clipboard"}</span
          >
        </button>
      </div>
    </div>
  </div>
{/if}
