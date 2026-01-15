<script lang="ts">
  import { authStore } from "$lib/stores/auth.svelte";
  import IconLayoutDashboard from "~icons/lucide/layout-dashboard";
</script>

{#if authStore.isAuthenticated}
  <a
    href="/admin"
    aria-label="Admin dashboard"
    class="relative size-9 rounded-md border border-zinc-300 dark:border-zinc-700 bg-zinc-100 dark:bg-zinc-900/50 hover:bg-zinc-200 dark:hover:bg-zinc-800/70 transition-all duration-200 cursor-pointer flex items-center justify-center"
  >
    <IconLayoutDashboard class="size-5 text-zinc-600 dark:text-zinc-400" />
  </a>
{/if}
