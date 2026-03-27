<script lang="ts">
  import Input from "$lib/components/admin/Input.svelte";
  import EventLog from "$lib/components/admin/EventLog.svelte";
  import { getAdminEvents } from "$lib/api";
  import { getLogger } from "@logtape/logtape";
  import type { AdminEvent } from "$lib/admin-types";

  const logger = getLogger(["admin", "events"]);
  import { css } from "styled-system/css";
  import { grid } from "styled-system/patterns";
  import {
    pageTitleClass,
    pageDescriptionClass,
    adminCardClass,
  } from "$lib/styles/admin";

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
      logger.error("Failed to load events", { error });
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

<div class={css({ spaceY: "6" })}>
  <!-- Header -->
  <div>
    <h1 class={pageTitleClass}>Event Log</h1>
    <p class={pageDescriptionClass}>
      System activity, errors, and sync operations
    </p>
  </div>

  <!-- Filters -->
  <div class={adminCardClass}>
    <h3
      class={css({
        fontSize: "sm",
        fontWeight: "medium",
        color: "admin.textSecondary",
        mb: "4",
      })}
    >
      Filters
    </h3>
    <div class={grid({ columns: { md: 2 }, gap: "4" })}>
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
    <div
      class={css({ textAlign: "center", py: "12", color: "admin.textMuted" })}
    >
      Loading events...
    </div>
  {:else if events.length === 0}
    <div class={css({ textAlign: "center", py: "12" })}>
      <p class={css({ color: "admin.textMuted" })}>No events found</p>
    </div>
  {:else}
    <div
      class={css({
        rounded: "xl",
        borderWidth: "1px",
        borderColor: "admin.border",
        bg: "admin.surface/50",
        overflow: "hidden",
        shadow: "sm",
        shadowColor: "black/10",
        _dark: { shadowColor: "black/20" },
      })}
    >
      <div
        class={css({
          px: "6",
          py: "3.5",
          bg: "admin.surfaceHover/30",
          borderBottomWidth: "1px",
          borderColor: "admin.border",
        })}
      >
        <h2
          class={css({
            fontSize: "sm",
            fontWeight: "medium",
            color: "admin.textSecondary",
          })}
        >
          Event Log
          <span
            class={css({
              color: "admin.textMuted",
              fontWeight: "normal",
              ml: "2",
            })}
          >
            ({events.length} event{events.length === 1 ? "" : "s"})
          </span>
        </h2>
      </div>
      <EventLog {events} maxHeight="600px" showMetadata={true} />
    </div>
  {/if}
</div>
