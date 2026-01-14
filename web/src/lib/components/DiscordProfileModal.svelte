<script lang="ts">
  import { fade, scale } from "svelte/transition";
  import IconCopy from "~icons/material-symbols/content-copy-rounded";
  import IconCheck from "~icons/material-symbols/check-rounded";

  interface Props {
    open: boolean;
    username: string;
    avatarUrl?: string;
    bannerUrl?: string;
  }

  let {
    open = $bindable(false),
    username,
    avatarUrl = "https://cdn.discordapp.com/avatars/184118083143598081/798e497f55abdcadbd8440e5eed551a0.png?size=4096",
    bannerUrl = "https://cdn.discordapp.com/banners/184118083143598081/174425460b67261a124d873b016e038f.png?size=4096",
  }: Props = $props();

  let copySuccess = $state(false);
  let avatarFailed = $state(false);
  let bannerFailed = $state(false);

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      open = false;
    }
  }

  function handleClose() {
    open = false;
  }

  async function copyUsername() {
    try {
      await navigator.clipboard.writeText(username);
      copySuccess = true;
      setTimeout(() => {
        copySuccess = false;
      }, 2000);
    } catch (err) {
      console.error("Failed to copy username:", err);
    }
  }
</script>

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-start justify-center bg-black/30 backdrop-blur-[2px] p-4 pt-[15vh]"
    onclick={handleBackdropClick}
    onkeydown={(e) => e.key === "Escape" && handleClose()}
    role="presentation"
    tabindex="-1"
    transition:fade={{ duration: 200 }}
  >
    <!-- SCALE: Adjust the scale() value to resize entire modal proportionally -->
    <div
      class="relative w-full max-w-md rounded-xl bg-zinc-100 dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700 shadow-lg overflow-hidden scale-110 origin-top"
      role="dialog"
      aria-modal="true"
      aria-labelledby="discord-profile-title"
      transition:scale={{ duration: 200, start: 0.95 }}
    >
      <!-- Banner -->
      {#if bannerUrl && !bannerFailed}
        <img
          src={bannerUrl}
          alt=""
          class="h-28 w-full object-cover"
          onerror={() => (bannerFailed = true)}
        />
      {:else}
        <div
          class="h-28 bg-linear-to-br from-zinc-300 to-zinc-400 dark:from-zinc-700 dark:to-zinc-800"
        ></div>
      {/if}

      <!-- Content area -->
      <div class="px-5 pb-5">
        <!-- Avatar with stroke effect -->
        <div class="relative -mt-14 mb-3 w-fit">
          <!-- Stroke ring (larger circle behind avatar) -->
          <!-- SIZE: avatar (96px) + stroke (4px * 2) = 104px -->
          <!-- POSITION: -m-1 centers the stroke ring behind the avatar -->
          <div
            class="absolute inset-0 -m-1 size-[104px] rounded-full bg-zinc-100 dark:bg-zinc-900"
          ></div>

          <!-- Avatar circle -->
          <!-- SIZE: size-24 = 96px -->
          {#if avatarUrl && !avatarFailed}
            <img
              src={avatarUrl}
              alt="Profile avatar"
              class="relative size-24 rounded-full object-cover"
              onerror={() => (avatarFailed = true)}
            />
          {:else}
            <div
              class="relative size-24 rounded-full bg-linear-to-br from-zinc-400 to-zinc-500 dark:from-zinc-500 dark:to-zinc-600"
            ></div>
          {/if}

          <!-- Online indicator -->
          <!-- POSITION: bottom/right values place center on avatar circumference -->
          <!-- For 96px avatar at 315° (bottom-right): ~4px from edge -->
          <div
            class="absolute bottom-0.5 right-0.5 size-5 rounded-full bg-green-500 border-[3px] border-zinc-100 dark:border-zinc-900"
          ></div>
        </div>

        <!-- Profile info -->
        <!-- SPACING: mb-4 controls gap before About Me section -->
        <div class="mb-4">
          <h2
            id="discord-profile-title"
            class="text-xl font-bold text-zinc-900 dark:text-zinc-100"
          >
            Xevion
          </h2>
          <!-- USERNAME ROW: gap-1.5 controls spacing between elements -->
          <div class="flex items-center gap-1.5 text-sm">
            <span
              class="font-mono text-xs px-1.5 py-0.5 rounded border border-zinc-300 dark:border-zinc-700 bg-zinc-200/50 dark:bg-zinc-800/50 text-zinc-600 dark:text-zinc-400"
              >{username}</span
            >
            <button
              onclick={copyUsername}
              class="p-0.5 rounded hover:bg-zinc-200 dark:hover:bg-zinc-800 transition-colors"
              title={copySuccess ? "Copied!" : "Copy username"}
            >
              {#if copySuccess}
                <IconCheck
                  class="size-3.5 text-green-600 dark:text-green-500"
                />
              {:else}
                <IconCopy class="size-3.5 text-zinc-400 dark:text-zinc-500" />
              {/if}
            </button>
            <span class="text-zinc-400 dark:text-zinc-500">·</span>
            <span class="text-zinc-500 dark:text-zinc-400">any/they</span>
          </div>
        </div>

        <!-- About Me section -->
        <div
          class="p-3 rounded-lg bg-zinc-200/50 dark:bg-zinc-800/50 border border-zinc-200 dark:border-zinc-700"
        >
          <h3
            class="text-xs font-semibold uppercase text-zinc-500 dark:text-zinc-500 mb-1"
          >
            About Me
          </h3>
          <p class="text-sm text-zinc-700 dark:text-zinc-300">
            Live with dignity.<br />
            <a
              href="https://xevion.dev"
              class="text-blue-600 dark:text-blue-400 hover:underline"
              target="_blank"
              rel="noopener noreferrer">https://xevion.dev</a
            >
          </p>
        </div>
      </div>
    </div>
  </div>
{/if}
