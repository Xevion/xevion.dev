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
    scrollbars: { autoHide: "leave", autoHideDelay: 800 }
  }}
  defer
  style="max-height: {maxHeight}"
>
  <div class="divide-y divide-zinc-800/50 bg-zinc-950">
    {#each events as event}
      {@const levelColors = {
        info: "text-cyan-500/60",
        warning: "text-amber-500/70",
        error: "text-rose-500/70"
      }}
      {@const levelLabels = {
        info: "INFO",
        warning: "WARN",
        error: "ERR"
      }}
      <div class="hover:bg-zinc-900/50 transition-colors">
        <div class="px-4 py-1.5">
          <div class="flex items-center justify-between gap-4 text-xs">
            <div class="flex items-center gap-2.5 flex-1 min-w-0">
              <span class={`${levelColors[event.level]} font-mono font-medium shrink-0 w-10`}>
                {levelLabels[event.level]}
              </span>
              <span class="text-zinc-300 truncate">
                {event.message}
              </span>
              <span class="text-zinc-500 shrink-0">
                target=<span class="text-zinc-400">{event.target}</span>
              </span>
            </div>
            <div class="flex items-center gap-3 shrink-0">
              {#if showMetadata && event.metadata}
                <button
                  class="text-[11px] text-indigo-400 hover:text-indigo-300 transition-colors"
                  onclick={() => toggleMetadata(event.id)}
                >
                  {expandedEventId === event.id ? "hide" : "show"}
                </button>
              {/if}
              <span class="text-zinc-600 text-[11px] tabular-nums">
                {formatTimestamp(event.timestamp)}
              </span>
            </div>
          </div>
        </div>
        {#if showMetadata && expandedEventId === event.id && event.metadata}
          <div class="px-4 pb-2">
            <div class="bg-zinc-900 border border-zinc-800 rounded p-3 text-[11px]">
              <p class="text-zinc-500 mb-2 font-medium">Metadata:</p>
              <pre class="text-zinc-400 overflow-x-auto">{JSON.stringify(event.metadata, null, 2)}</pre>
            </div>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</OverlayScrollbarsComponent>
