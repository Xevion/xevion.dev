<script lang="ts">
  import { themeStore } from "$lib/stores/theme.svelte";
  import IconSun from "~icons/lucide/sun";
  import IconMoon from "~icons/lucide/moon";
</script>

<button
  type="button"
  onclick={() => themeStore.toggle()}
  aria-label={themeStore.isDark
    ? "Switch to light mode"
    : "Switch to dark mode"}
  class="relative size-9 rounded-md border border-zinc-300 dark:border-zinc-700 bg-zinc-100 dark:bg-zinc-900/50 hover:bg-zinc-200 dark:hover:bg-zinc-800/70 transition-all duration-200"
>
  <div class="absolute inset-0 flex items-center justify-center">
    <IconSun
      class="size-5 text-zinc-600 dark:text-zinc-400 transition-all duration-300 {themeStore.isDark
        ? 'rotate-90 scale-0 opacity-0'
        : 'rotate-0 scale-100 opacity-100'}"
    />
    <IconMoon
      class="absolute size-5 text-zinc-600 dark:text-zinc-400 transition-all duration-300 {themeStore.isDark
        ? 'rotate-0 scale-100 opacity-100'
        : '-rotate-90 scale-0 opacity-0'}"
    />
  </div>
</button>
