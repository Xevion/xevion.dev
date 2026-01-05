<script lang="ts">
  import AppWrapper from "$lib/components/AppWrapper.svelte";
  import IconSimpleIconsGithub from "~icons/simple-icons/github";
  import IconSimpleIconsLinkedin from "~icons/simple-icons/linkedin";
  import IconSimpleIconsDiscord from "~icons/simple-icons/discord";
  import MaterialSymbolsMailRounded from "~icons/material-symbols/mail-rounded";
  import MaterialSymbolsVpnKey from "~icons/material-symbols/vpn-key";
</script>

<AppWrapper class="overflow-x-hidden font-schibsted">
  <!-- Top Navigation Bar -->
  <div class="flex w-full justify-end items-center pt-5 px-6 pb-9">
    <!-- <div class="flex gap-4 items-center"></div> -->
  </div>

  <!-- Main Content -->
  <div class="flex items-center flex-col">
    <div
      class="max-w-2xl mx-6 border-b border-zinc-700 divide-y divide-zinc-700"
    >
      <!-- Name & Occupation -->
      <div class="flex flex-col pb-4">
        <span class="text-3xl font-bold text-white">Ryan Walters,</span>
        <span class="text-2xl font-normal text-zinc-400">
          Full-Stack Software Engineer
        </span>
      </div>

      <div class="py-4 text-zinc-200">
        <p class="text-[0.95em]">
          A fanatical software engineer with expertise and passion for sound,
          scalable and high-performance applications. I'm always working on
          something new. <br />
          Sometimes innovative &mdash; sometimes crazy.
        </p>
      </div>

      <div class="py-3">
        <span class="text-zinc-200">Connect with me</span>
        <div class="flex gap-x-2 pl-3 pt-3 pb-2">
          <a
            href="https://github.com/Xevion"
            class="flex items-center gap-x-1.5 px-1.5 py-1 rounded-sm bg-zinc-900 shadow-sm hover:bg-zinc-800 transition-colors"
          >
            <IconSimpleIconsGithub class="size-4 text-zinc-300" />
            <span class="text-sm text-zinc-100">GitHub</span>
          </a>
          <a
            href="https://linkedin.com/in/ryancwalters"
            class="flex items-center gap-x-1.5 px-1.5 py-1 rounded-sm bg-zinc-900 shadow-sm hover:bg-zinc-800 transition-colors"
          >
            <IconSimpleIconsLinkedin class="size-4 text-zinc-300" />
            <span class="text-sm text-zinc-100">LinkedIn</span>
          </a>
          <button
            type="button"
            class="flex items-center gap-x-1.5 px-1.5 py-1 rounded-sm bg-zinc-900 shadow-sm hover:bg-zinc-800 transition-colors"
          >
            <IconSimpleIconsDiscord class="size-4 text-zinc-300" />
            <span class="text-sm text-zinc-100">Discord</span>
          </button>
          <a
            href="mailto:your.email@example.com"
            class="flex items-center gap-x-1.5 px-1.5 py-1 rounded-sm bg-zinc-900 shadow-sm hover:bg-zinc-800 transition-colors"
          >
            <MaterialSymbolsMailRounded class="size-4.5 text-zinc-300" />
            <span class="text-sm text-zinc-100">Email</span>
          </a>
          <button
            type="button"
            class="flex items-center gap-x-1.5 px-1.5 py-1 rounded-sm bg-zinc-900 shadow-sm hover:bg-zinc-800 transition-colors"
          >
            <MaterialSymbolsVpnKey class="size-4.5 text-zinc-300" />
            <span class="text-sm text-zinc-100">PGP Key</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</AppWrapper>
