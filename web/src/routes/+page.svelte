<script lang="ts">
  import AppWrapper from "$lib/components/AppWrapper.svelte";
  import ProjectCard from "$lib/components/ProjectCard.svelte";
  import DiscordProfileModal from "$lib/components/DiscordProfileModal.svelte";
  import type { PageData } from "./$types";
  import MaterialSymbolsVpnKey from "~icons/material-symbols/vpn-key";

  interface ExtendedPageData extends PageData {
    socialLinksWithIcons: Array<{
      id: string;
      platform: string;
      label: string;
      value: string;
      icon: string;
      iconSvg: string;
      visible: boolean;
      displayOrder: number;
    }>;
  }

  let { data }: { data: ExtendedPageData } = $props();
  const projects = $derived(data.projects);
  const socialLinksWithIcons = $derived(data.socialLinksWithIcons);

  // Filter visible social links
  const visibleSocialLinks = $derived(
    socialLinksWithIcons.filter((link: { visible: boolean }) => link.visible),
  );

  let discordModalOpen = $state(false);
  let discordUsername = $state("");
</script>

<AppWrapper class="overflow-x-hidden font-schibsted">
  <div class="flex items-center flex-col pt-14">
    <div
      class="max-w-2xl mx-4 border-b border-zinc-200 dark:border-zinc-700 divide-y divide-zinc-200 dark:divide-zinc-700 sm:mx-6"
    >
      <div class="flex flex-col pb-4">
        <span
          class="text-2xl font-bold text-zinc-900 dark:text-white sm:text-3xl"
          >{data.settings.identity.displayName},</span
        >
        <span
          class="text-xl font-normal text-zinc-600 dark:text-zinc-400 sm:text-2xl"
        >
          {data.settings.identity.occupation}
        </span>
      </div>

      <div class="py-4 text-zinc-700 dark:text-zinc-200">
        <p class="sm:text-[0.95em] whitespace-pre-line">
          {data.settings.identity.bio}
        </p>
      </div>

      <div class="py-3">
        <span class="text-zinc-700 dark:text-zinc-200">Connect with me</span>
        <div class="flex flex-wrap gap-2 pl-3 pt-3 pb-2">
          {#each visibleSocialLinks as link (link.id)}
            {#if link.platform === "github" || link.platform === "linkedin"}
              <!-- Simple link platforms -->
              <a
                href={link.value}
                class="flex items-center gap-x-1.5 px-1.5 py-1 rounded-sm bg-zinc-100 dark:bg-zinc-900 shadow-sm hover:bg-zinc-200 dark:hover:bg-zinc-800 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-zinc-400 dark:focus-visible:ring-zinc-500"
              >
                <span class="size-4 text-zinc-600 dark:text-zinc-300">
                  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                  {@html link.iconSvg}
                </span>
                <span
                  class="whitespace-nowrap text-sm text-zinc-800 dark:text-zinc-100"
                  >{link.label}</span
                >
              </a>
            {:else if link.platform === "discord"}
              <!-- Discord - button that opens profile modal -->
              <button
                type="button"
                class="flex items-center gap-x-1.5 px-1.5 py-1 rounded-sm bg-zinc-100 dark:bg-zinc-900 shadow-sm hover:bg-zinc-200 dark:hover:bg-zinc-800 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-zinc-400 dark:focus-visible:ring-zinc-500"
                onclick={() => {
                  discordUsername = link.value;
                  discordModalOpen = true;
                }}
              >
                <span class="size-4 text-zinc-600 dark:text-zinc-300">
                  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                  {@html link.iconSvg}
                </span>
                <span
                  class="whitespace-nowrap text-sm text-zinc-800 dark:text-zinc-100"
                  >{link.label}</span
                >
              </button>
            {:else if link.platform === "email"}
              <!-- Email - mailto link -->
              <a
                href="mailto:{link.value}"
                class="flex items-center gap-x-1.5 px-1.5 py-1 rounded-sm bg-zinc-100 dark:bg-zinc-900 shadow-sm hover:bg-zinc-200 dark:hover:bg-zinc-800 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-zinc-400 dark:focus-visible:ring-zinc-500"
              >
                <span class="size-4.5 text-zinc-600 dark:text-zinc-300">
                  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                  {@html link.iconSvg}
                </span>
                <span
                  class="whitespace-nowrap text-sm text-zinc-800 dark:text-zinc-100"
                  >{link.label}</span
                >
              </a>
            {/if}
          {/each}
          <!-- PGP Key - links to dedicated page -->
          <a
            href="/pgp"
            class="flex items-center gap-x-1.5 px-1.5 py-1 rounded-sm bg-zinc-100 dark:bg-zinc-900 shadow-sm hover:bg-zinc-200 dark:hover:bg-zinc-800 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-zinc-400 dark:focus-visible:ring-zinc-500"
          >
            <MaterialSymbolsVpnKey
              class="size-4.5 text-zinc-600 dark:text-zinc-300"
            />
            <span
              class="whitespace-nowrap text-sm text-zinc-800 dark:text-zinc-100"
              >PGP Key</span
            >
          </a>
        </div>
      </div>
    </div>

    <div class="max-w-2xl mx-4 mt-5 sm:mx-6">
      <div class="grid grid-cols-1 gap-2.5 sm:grid-cols-2">
        {#each projects as project (project.id)}
          <ProjectCard {project} />
        {/each}
      </div>
    </div>
  </div>
</AppWrapper>

<DiscordProfileModal bind:open={discordModalOpen} username={discordUsername} />
