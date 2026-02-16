<script lang="ts">
  import { resolve } from "$app/paths";
  import Button from "$lib/components/admin/Button.svelte";
  import EventLog from "$lib/components/admin/EventLog.svelte";
  import { getAdminEvents } from "$lib/api";
  import type { AdminEvent } from "$lib/admin-types";
  import IconPlus from "~icons/lucide/plus";
  import { css, cx } from "styled-system/css";
  import { wrap, hstack } from "styled-system/patterns";
  import {
    pageTitleClass,
    pageDescriptionClass,
    iconSm,
  } from "$lib/styles/admin";

  let recentEvents = $state<AdminEvent[]>([]);
  let loading = $state(true);

  async function loadDashboard() {
    try {
      const eventsData = await getAdminEvents();
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

<div class={css({ spaceY: "6" })}>
  <!-- Header -->
  <div>
    <h1 class={pageTitleClass}>Dashboard</h1>
    <p class={cx(pageDescriptionClass, css({ mt: "1" }))}>
      Overview of your portfolio and recent activity
    </p>
  </div>

  {#if loading}
    <div
      class={css({ textAlign: "center", py: "12", color: "admin.textMuted" })}
    >
      Loading dashboard...
    </div>
  {:else}
    <!-- Quick Actions -->
    <div class={wrap({ gap: "3" })}>
      <Button variant="primary" href="/admin/projects/new">
        <IconPlus class={cx(iconSm, css({ mr: "2" }))} />
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
      class={css({
        rounded: "xl",
        borderWidth: "1px",
        borderColor: "admin.border",
        bg: "admin.surface",
        overflow: "hidden",
        shadow: "sm",
        shadowColor: "black/10",
        _dark: { shadowColor: "black/20" },
      })}
    >
      <div
        class={hstack({
          justify: "space-between",
          gap: "0",
          px: "6",
          py: "3.5",
          bg: "admin.surfaceHover",
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
          Recent Events
        </h2>
        <a
          href={resolve("/admin/events")}
          class={css({
            fontSize: "sm",
            color: "admin.accent",
            _hover: { color: "admin.accentHover" },
            transition: "colors",
          })}
        >
          View all →
        </a>
      </div>

      {#if recentEvents.length === 0}
        <p
          class={css({
            fontSize: "sm",
            color: "admin.textMuted",
            textAlign: "center",
            py: "8",
          })}
        >
          No events yet
        </p>
      {:else}
        <EventLog events={recentEvents} maxHeight="400px" />
      {/if}
    </div>
  {/if}
</div>
