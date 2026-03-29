<script lang="ts">
  import Input from "$lib/components/admin/Input.svelte";
  import EventLog from "$lib/components/admin/EventLog.svelte";
  import { getAdminEvents } from "$lib/api";
  import { getLogger } from "@logtape/logtape";
  import type { ApiEvent, EventLevel } from "$lib/bindings";

  const logger = getLogger(["admin", "events"]);
  import { css } from "styled-system/css";
  import { grid } from "styled-system/patterns";
  import {
    pageTitleClass,
    pageDescriptionClass,
    adminCardClass,
  } from "$lib/styles/admin";

  const PAGE_SIZE = 100;

  let events = $state<ApiEvent[]>([]);
  let loading = $state(true);
  let loadingMore = $state(false);
  let hasMore = $state(true);
  let filterLevel = $state<string>("");
  let filterEntityType = $state<string>("");
  let filterEventType = $state<string>("");

  const levelOptions = [
    { value: "", label: "All Levels" },
    { value: "info", label: "Info" },
    { value: "warning", label: "Warning" },
    { value: "error", label: "Error" },
  ];

  const entityTypeOptions = [
    { value: "", label: "All Entities" },
    { value: "project", label: "Project" },
    { value: "tag", label: "Tag" },
    { value: "settings", label: "Settings" },
    { value: "system", label: "System" },
  ];

  const eventTypeOptions = [
    { value: "", label: "All Types" },
    { value: "project.created", label: "Project Created" },
    { value: "project.updated", label: "Project Updated" },
    { value: "project.deleted", label: "Project Deleted" },
    { value: "project.tag_added", label: "Tag Added to Project" },
    { value: "project.tag_removed", label: "Tag Removed from Project" },
    { value: "tag.created", label: "Tag Created" },
    { value: "tag.updated", label: "Tag Updated" },
    { value: "tag.deleted", label: "Tag Deleted" },
    { value: "settings.updated", label: "Settings Updated" },
    { value: "github.sync_completed", label: "GitHub Sync" },
    { value: "github.sync_failed", label: "GitHub Sync Failed" },
    { value: "github.rate_limited", label: "GitHub Rate Limited" },
    { value: "og.generated", label: "OG Image Generated" },
    { value: "og.failed", label: "OG Image Failed" },
    { value: "cache.invalidated", label: "Cache Invalidated" },
  ];

  async function loadEvents(reset = true) {
    if (reset) {
      loading = true;
      events = [];
    } else {
      loadingMore = true;
    }
    try {
      const result = await getAdminEvents({
        limit: PAGE_SIZE,
        offset: reset ? 0 : events.length,
        level: (filterLevel || undefined) as EventLevel | undefined,
        entityType: filterEntityType || undefined,
        eventType: filterEventType || undefined,
      });
      if (reset) {
        events = result;
      } else {
        events = [...events, ...result];
      }
      hasMore = result.length === PAGE_SIZE;
    } catch (error) {
      logger.error("Failed to load events", { error });
    } finally {
      loading = false;
      loadingMore = false;
    }
  }

  // Reload when filters change
  $effect(() => {
    void filterLevel;
    void filterEntityType;
    void filterEventType;
    loadEvents(true);
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
      System activity, content changes, and background operations
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
    <div class={grid({ columns: { md: 3 }, gap: "4" })}>
      <Input
        label="Level"
        type="select"
        bind:value={filterLevel}
        options={levelOptions}
      />
      <Input
        label="Entity Type"
        type="select"
        bind:value={filterEntityType}
        options={entityTypeOptions}
      />
      <Input
        label="Event Type"
        type="select"
        bind:value={filterEventType}
        options={eventTypeOptions}
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
            ({events.length} event{events.length === 1 ? "" : "s"}{hasMore
              ? "+"
              : ""})
          </span>
        </h2>
      </div>
      <EventLog {events} maxHeight="600px" showMetadata={true} />
      {#if hasMore}
        <div
          class={css({
            px: "6",
            py: "3",
            borderTopWidth: "1px",
            borderColor: "admin.border",
            textAlign: "center",
          })}
        >
          <button
            class={css({
              fontSize: "sm",
              color: "admin.accent",
              _hover: { color: "admin.accentHover" },
              cursor: "pointer",
              transition: "colors",
            })}
            disabled={loadingMore}
            onclick={() => loadEvents(false)}
          >
            {loadingMore ? "Loading..." : "Load more"}
          </button>
        </div>
      {/if}
    </div>
  {/if}
</div>
