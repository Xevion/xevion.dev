<script lang="ts">
  import Input from "$lib/components/admin/Input.svelte";
  import EventLog from "$lib/components/admin/EventLog.svelte";
  import { getAdminEvents } from "$lib/api";
  import type { AdminEvent } from "$lib/admin-types";

  let events = $state<AdminEvent[]>([]);
  let loading = $state(true);
  let filterLevel = $state<string>("");
  let filterTarget = $state("");

  const levelOptions = [
    { value: "", label: "All Levels" },
    { value: "info", label: "Info" },
    { value: "warning", label: "Warning" },
    { value: "error", label: "Error" },
  ];

  async function loadEvents() {
    loading = true;
    try {
      // TODO: Pass filters when backend implementation is complete
      events = await getAdminEvents();
    } catch (error) {
      console.error("Failed to load events:", error);
    } finally {
      loading = false;
    }
  }

  // Load events on mount and when filters change
  $effect(() => {
    void filterLevel;
    void filterTarget;
    loadEvents();
  });
</script>

<svelte:head>
  <title>Events | Admin</title>
</svelte:head>

<div class="space-y-6">
  <!-- Header -->
  <div>
    <h1 class="text-xl font-semibold text-admin-text">Event Log</h1>
    <p class="mt-1 text-sm text-admin-text-muted">
      System activity, errors, and sync operations
    </p>
  </div>

  <!-- Filters -->
  <div
    class="rounded-xl border border-admin-border bg-admin-surface p-6 shadow-sm shadow-black/10 dark:shadow-black/20"
  >
    <h3 class="text-sm font-medium text-admin-text-secondary mb-4">Filters</h3>
    <div class="grid gap-4 md:grid-cols-2">
      <Input
        label="Level"
        type="select"
        bind:value={filterLevel}
        options={levelOptions}
      />
      <Input
        label="Target"
        type="text"
        bind:value={filterTarget}
        placeholder="e.g., project, tag, github"
      />
    </div>
  </div>

  <!-- Events Log -->
  {#if loading}
    <div class="text-center py-12 text-admin-text-muted">Loading events...</div>
  {:else if events.length === 0}
    <div class="text-center py-12">
      <p class="text-admin-text-muted">No events found</p>
    </div>
  {:else}
    <div
      class="rounded-xl border border-admin-border bg-admin-surface/50 overflow-hidden shadow-sm shadow-black/10 dark:shadow-black/20"
    >
      <div
        class="px-6 py-3.5 bg-admin-surface-hover/30 border-b border-admin-border"
      >
        <h2 class="text-sm font-medium text-admin-text-secondary">
          Event Log
          <span class="text-admin-text-muted font-normal ml-2">
            ({events.length} event{events.length === 1 ? "" : "s"})
          </span>
        </h2>
      </div>
      <EventLog {events} maxHeight="600px" showMetadata={true} />
    </div>
  {/if}
</div>
