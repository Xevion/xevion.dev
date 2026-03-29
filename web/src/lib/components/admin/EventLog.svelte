<script lang="ts">
  import { css, cx } from "styled-system/css";
  import { hstack } from "styled-system/patterns";
  import type { ApiEvent } from "$lib/bindings";
  import { OverlayScrollbarsComponent } from "overlayscrollbars-svelte";
  import "overlayscrollbars/overlayscrollbars.css";

  interface Props {
    events: ApiEvent[];
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

  const levelColorStyles = {
    info: css({ color: "cyan.500/60" }),
    warning: css({ color: "amber.500/70" }),
    error: css({ color: "rose.500/70" }),
  };

  const levelLabels = {
    info: "INFO",
    warning: "WARN",
    error: "ERR",
  };
</script>

<OverlayScrollbarsComponent
  options={{
    scrollbars: { autoHide: "leave", autoHideDelay: 800 },
  }}
  defer
  style="max-height: {maxHeight}"
>
  <div
    class={css({
      divideY: "1px",
      divideColor: "admin.border/50",
      bg: "admin.bg",
    })}
  >
    {#each events as event (event.id)}
      <div
        class={css({
          _hover: { bg: "admin.surfaceHover/50" },
          transition: "colors",
        })}
      >
        <div class={css({ px: "4", py: "1.5" })}>
          <div
            class={hstack({
              justify: "space-between",
              gap: "4",
              fontSize: "xs",
            })}
          >
            <div class={hstack({ gap: "2.5", flex: "1", minW: "0" })}>
              <span
                class={cx(
                  levelColorStyles[event.level],
                  css({
                    fontFamily: "mono",
                    fontWeight: "medium",
                    flexShrink: "0",
                    w: "10",
                  }),
                )}
              >
                {levelLabels[event.level]}
              </span>
              <span
                class={css({
                  color: "admin.textMuted",
                  fontFamily: "mono",
                  fontSize: "11px",
                  flexShrink: "0",
                  w: "40",
                })}
              >
                {event.eventType}
              </span>
              <span
                class={css({
                  color: "admin.text",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  whiteSpace: "nowrap",
                })}
              >
                {event.message}
              </span>
            </div>
            <div class={hstack({ gap: "3", flexShrink: "0" })}>
              {#if showMetadata && event.metadata}
                <button
                  class={css({
                    fontSize: "11px",
                    color: "admin.accent",
                    _hover: { color: "admin.accentHover" },
                    transition: "colors",
                  })}
                  onclick={() => toggleMetadata(event.id)}
                >
                  {expandedEventId === event.id ? "hide" : "meta"}
                </button>
              {/if}
              {#if event.actor}
                <span
                  class={css({
                    color: "admin.textMuted",
                    fontSize: "11px",
                  })}
                >
                  {event.actor}
                </span>
              {/if}
              <span
                class={css({
                  color: "admin.textMuted",
                  fontSize: "11px",
                  fontVariantNumeric: "tabular-nums",
                })}
              >
                {formatTimestamp(event.createdAt)}
              </span>
            </div>
          </div>
        </div>
        {#if showMetadata && expandedEventId === event.id && event.metadata}
          <div class={css({ px: "4", pb: "2" })}>
            <div
              class={css({
                bg: "admin.surface",
                borderWidth: "1px",
                borderColor: "admin.border",
                rounded: "sm",
                p: "3",
                fontSize: "11px",
              })}
            >
              <pre
                class={css({
                  color: "admin.textSecondary",
                  overflowX: "auto",
                })}>{JSON.stringify(event.metadata, null, 2)}</pre>
            </div>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</OverlayScrollbarsComponent>
