<script lang="ts">
  import type { AdminEvent } from "$lib/admin-types";
  import { OverlayScrollbarsComponent } from "overlayscrollbars-svelte";
  import "overlayscrollbars/overlayscrollbars.css";

  interface Props {
    events: AdminEvent[];
    maxHeight?: string;
    showMetadata?: boolean;
  }

  let { events, maxHeight = "400px", showMetadata = false }: Props = $props();

  let expandedEventId = $state<string | null>(null);

  function formatTimestamp(timestamp: string): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return "just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    return `${diffDays}d ago`;
  }

  function toggleMetadata(eventId: string) {
    expandedEventId = expandedEventId === eventId ? null : eventId;
  }
</script>

<OverlayScrollbarsComponent
  options={{
    scrollbars: { autoHide: "leave", autoHideDelay: 800 },
  }}
  defer
  style="max-height: {maxHeight}"
>
  <div class="divide-y divide-admin-border/50 bg-admin-bg">
    {#each events as event (event.id)}
      {@const levelColors = {
        info: "text-cyan-500/60",
        warning: "text-amber-500/70",
        error: "text-rose-500/70",
      }}
      {@const levelLabels = {
        info: "INFO",
        warning: "WARN",
        error: "ERR",
      }}
      <div class="hover:bg-admin-surface-hover/50 transition-colors">
        <div class="px-4 py-1.5">
          <div class="flex items-center justify-between gap-4 text-xs">
            <div class="flex items-center gap-2.5 flex-1 min-w-0">
              <span
                class={`${levelColors[event.level]} font-mono font-medium shrink-0 w-10`}
              >
                {levelLabels[event.level]}
              </span>
              <span class="text-admin-text truncate">
                {event.message}
              </span>
              <span class="text-admin-text-muted shrink-0">
                target=<span class="text-admin-text-secondary">{event.target}</span>
              </span>
            </div>
            <div class="flex items-center gap-3 shrink-0">
              {#if showMetadata && event.metadata}
                <button
                  class="text-[11px] text-admin-accent hover:text-admin-accent-hover transition-colors"
                  onclick={() => toggleMetadata(event.id)}
                >
                  {expandedEventId === event.id ? "hide" : "show"}
                </button>
              {/if}
              <span class="text-admin-text-muted text-[11px] tabular-nums">
                {formatTimestamp(event.timestamp)}
              </span>
            </div>
          </div>
        </div>
        {#if showMetadata && expandedEventId === event.id && event.metadata}
          <div class="px-4 pb-2">
            <div
              class="bg-admin-surface border border-admin-border rounded p-3 text-[11px]"
            >
              <p class="text-admin-text-muted mb-2 font-medium">Metadata:</p>
              <pre class="text-admin-text-secondary overflow-x-auto">{JSON.stringify(
                  event.metadata,
                  null,
                  2,
                )}</pre>
            </div>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</OverlayScrollbarsComponent>
