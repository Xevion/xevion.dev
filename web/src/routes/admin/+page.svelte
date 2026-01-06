<script lang="ts">
  import { resolve } from "$app/paths";
  import Button from "$lib/components/admin/Button.svelte";
  import EventLog from "$lib/components/admin/EventLog.svelte";
  import { getAdminEvents } from "$lib/api";
  import type { AdminEvent } from "$lib/admin-types";
  import IconPlus from "~icons/lucide/plus";

  let recentEvents = $state<AdminEvent[]>([]);
  let loading = $state(true);

  async function loadDashboard() {
    try {
      const eventsData = await getAdminEvents({ limit: 10 });
      recentEvents = eventsData;
    } catch (error) {
      console.error("Failed to load dashboard:", error);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    loadDashboard();
  });
</script>

<svelte:head>
  <title>Dashboard | Admin</title>
</svelte:head>

<div class="space-y-6">
  <!-- Header -->
  <div>
    <h1 class="text-xl font-semibold text-zinc-50">Dashboard</h1>
    <p class="mt-1 text-sm text-zinc-500">
      Overview of your portfolio and recent activity
    </p>
  </div>

  {#if loading}
    <div class="text-center py-12 text-admin-text-muted">
      Loading dashboard...
    </div>
  {:else}
    <!-- Quick Actions -->
    <div class="flex flex-wrap gap-3">
      <Button variant="primary" href="/admin/projects/new">
        <IconPlus class="w-4 h-4 mr-2" />
        New Project
      </Button>
      <Button variant="secondary" href="/admin/projects">
        View All Projects
      </Button>
      <Button variant="secondary" href="/admin/tags">Manage Tags</Button>
      <Button variant="secondary" href="/admin/events">View Events</Button>
    </div>

    <!-- Recent Events -->
    <div
      class="rounded-xl border border-zinc-800 bg-zinc-900/50 overflow-hidden shadow-sm shadow-black/20"
    >
      <div
        class="flex items-center justify-between px-6 py-3.5 bg-zinc-800/30 border-b border-zinc-800"
      >
        <h2 class="text-sm font-medium text-zinc-300">Recent Events</h2>
        <a
          href={resolve("/admin/events")}
          class="text-sm text-indigo-400 hover:text-indigo-300 transition-colors"
        >
          View all →
        </a>
      </div>

      {#if recentEvents.length === 0}
        <p class="text-sm text-zinc-500 text-center py-8">No events yet</p>
      {:else}
        <EventLog events={recentEvents} maxHeight="400px" />
      {/if}
    </div>
  {/if}
</div>
