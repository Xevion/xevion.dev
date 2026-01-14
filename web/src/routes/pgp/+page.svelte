<script lang="ts">
  import { OverlayScrollbarsComponent } from "overlayscrollbars-svelte";
  import IconDownload from "~icons/material-symbols/download-rounded";
  import IconCopy from "~icons/material-symbols/content-copy-rounded";
  import IconCheck from "~icons/material-symbols/check-rounded";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  let copySuccess = $state(false);
  let copyCommandSuccess = $state(false);

  async function copyToClipboard() {
    try {
      await navigator.clipboard.writeText(data.key.content);
      copySuccess = true;
      setTimeout(() => {
        copySuccess = false;
      }, 2000);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  }

  async function copyCommand() {
    try {
      await navigator.clipboard.writeText(
        "curl https://xevion.dev/pgp | gpg --import",
      );
      copyCommandSuccess = true;
      setTimeout(() => {
        copyCommandSuccess = false;
      }, 2000);
    } catch (err) {
      console.error("Failed to copy command:", err);
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

<svelte:head>
  <title>PGP Public Key - Ryan Walters</title>
  <meta
    name="description"
    content="Download or copy Ryan Walters' PGP public key"
  />
</svelte:head>

<main class="page-main overflow-x-hidden font-schibsted">
  <div class="flex items-center flex-col pt-14 pb-20 px-4 sm:px-6">
    <div class="max-w-2xl w-full">
      <!-- Header -->
      <div class="mb-6">
        <h1
          class="text-2xl sm:text-3xl font-bold text-zinc-900 dark:text-white mb-2"
        >
          PGP Public Key
        </h1>
        <p class="text-sm sm:text-base text-zinc-600 dark:text-zinc-400">
          Use this key to send me encrypted messages or verify my signed
          content.
        </p>
      </div>

      <!-- Fingerprint -->
      <div
        class="mb-6 p-3 sm:p-4 bg-zinc-100 dark:bg-zinc-800 rounded-lg border border-zinc-200 dark:border-zinc-700"
      >
        <div
          class="text-xs sm:text-sm font-semibold text-zinc-700 dark:text-zinc-300 mb-2"
        >
          Key Fingerprint
        </div>
        <div
          class="font-mono text-sm sm:text-base text-zinc-900 dark:text-zinc-100 break-all"
        >
          {data.key.fingerprint}
        </div>
        <div
          class="mt-3 pt-3 border-t border-zinc-200 dark:border-zinc-700 space-y-1"
        >
          <div class="text-xs sm:text-sm text-zinc-600 dark:text-zinc-400">
            <span class="font-medium">Key ID:</span>
            <span class="font-mono ml-2">{data.key.keyId}</span>
          </div>
          <div class="text-xs sm:text-sm text-zinc-600 dark:text-zinc-400">
            <span class="font-medium">Email:</span>
            <span class="ml-2">{data.key.email}</span>
          </div>
        </div>
      </div>

      <!-- Key Content Card -->
      <div
        class="mb-6 border border-zinc-200 dark:border-zinc-700 rounded-lg overflow-hidden bg-white dark:bg-zinc-900"
      >
        <div
          class="px-3 sm:px-4 py-2 sm:py-3 bg-zinc-50 dark:bg-zinc-800 border-b border-zinc-200 dark:border-zinc-700"
        >
          <div
            class="text-xs sm:text-sm font-semibold text-zinc-700 dark:text-zinc-300"
          >
            Public Key
          </div>
        </div>
        <OverlayScrollbarsComponent
          options={{
            scrollbars: { autoHide: "leave", autoHideDelay: 800 },
          }}
          defer
          style="max-height: 400px"
        >
          <pre
            class="p-3 sm:p-4 text-xs font-mono text-zinc-800 dark:text-zinc-200 bg-zinc-50 dark:bg-zinc-900/50 overflow-x-auto">{data
              .key.content}</pre>
        </OverlayScrollbarsComponent>
      </div>

      <!-- Action Buttons -->
      <div class="flex flex-col sm:flex-row gap-2 sm:gap-3">
        <button
          onclick={copyToClipboard}
          class="flex items-center justify-center gap-2 px-3 sm:px-4 py-2 sm:py-2.5 rounded-sm bg-zinc-900 dark:bg-zinc-100 text-white dark:text-zinc-900 hover:bg-zinc-800 dark:hover:bg-zinc-200 transition-colors shadow-sm"
        >
          <IconCopy class="size-4 sm:size-5" />
          <span class="text-sm sm:text-base font-medium"
            >{copySuccess ? "Copied!" : "Copy to Clipboard"}</span
          >
        </button>
        <button
          onclick={downloadKey}
          class="flex items-center justify-center gap-2 px-3 sm:px-4 py-2 sm:py-2.5 rounded-sm bg-zinc-100 dark:bg-zinc-800 text-zinc-800 dark:text-zinc-100 hover:bg-zinc-200 dark:hover:bg-zinc-700 transition-colors"
        >
          <IconDownload class="size-4 sm:size-5" />
          <span class="text-sm sm:text-base font-medium">Download</span>
        </button>
      </div>

      <!-- Additional Info -->
      <div
        class="mt-8 p-3 sm:p-4 bg-zinc-50 dark:bg-zinc-800/50 rounded-lg border border-zinc-200 dark:border-zinc-700"
      >
        <h2
          class="text-xs sm:text-sm font-semibold text-zinc-700 dark:text-zinc-300 mb-2"
        >
          How to use this key
        </h2>
        <div
          class="text-xs sm:text-sm text-zinc-600 dark:text-zinc-400 space-y-2"
        >
          <p>
            Import this key into your GPG keyring to encrypt messages for me or
            verify my signatures:
          </p>
          <div class="relative">
            <pre
              class="p-2 sm:p-3 pr-12 bg-white dark:bg-zinc-900 rounded border border-zinc-200 dark:border-zinc-700 font-mono text-xs overflow-x-auto">curl https://xevion.dev/pgp | gpg --import</pre>
            <button
              onclick={copyCommand}
              disabled={copyCommandSuccess}
              class="absolute top-1/2 -translate-y-1/2 right-2 p-1 rounded border border-zinc-300 dark:border-zinc-600 bg-zinc-50 dark:bg-zinc-800 hover:bg-zinc-100 dark:hover:bg-zinc-700 hover:border-zinc-400 dark:hover:border-zinc-500 transition-all {copyCommandSuccess
                ? 'cursor-default'
                : 'cursor-pointer'}"
              title={copyCommandSuccess ? "Copied!" : "Copy command"}
            >
              {#if copyCommandSuccess}
                <IconCheck
                  class="size-3.5 text-green-600 dark:text-green-500"
                />
              {:else}
                <IconCopy class="size-3.5 text-zinc-600 dark:text-zinc-400" />
              {/if}
            </button>
          </div>
          <p class="text-xs text-zinc-500 dark:text-zinc-500">
            You can also find this key on public keyservers by searching for the
            fingerprint above.
          </p>
        </div>
      </div>
    </div>
  </div>
</main>
