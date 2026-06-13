<script lang="ts">
  import { onMount } from "svelte";
  import type { TerminalCast } from "$lib/bindings";

  // An asciinema-style session that types itself out, holds, and loops. The box
  // is a FIXED height: a hidden full transcript reserves the space and the
  // animated copy overlays it, so layout is stable from frame 0. Always dark.
  // Honors prefers-reduced-motion → renders the static transcript (no typing).
  interface Props {
    cast: TerminalCast;
    title?: string;
  }

  let { cast, title = "session" }: Props = $props();

  const lines = $derived(cast.lines);

  // Always-dark palette from the `terminal.*` design tokens (the terminal does
  // not follow the site light/dark theme). Referenced as CSS vars so the values
  // live in one place (panda.config) for both the static chrome and line colors.
  const term = {
    dim: "var(--colors-terminal-dim)",
    text: "var(--colors-terminal-text)",
    cmd: "var(--colors-terminal-cmd)",
    muted: "var(--colors-terminal-muted)",
    err: "var(--colors-terminal-err)",
  };

  function lineColor(t: string): string {
    if (t === "err") return term.err;
    if (t === "muted") return term.muted;
    if (t === "cmd") return term.cmd;
    return term.text;
  }

  // `step` counts revealed lines during animation. Until animation starts —
  // SSR, no-JS, reduced-motion — `reveal` falls back to the full transcript, so
  // the static state is the whole session (never a mid-animation empty box).
  let step = $state(0);
  let typed = $state(0);
  let animate = $state(false);
  const reveal = $derived(animate ? step : lines.length);

  onMount(() => {
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    animate = !mq.matches;
    const onChange = () => (animate = !mq.matches);
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  });

  $effect(() => {
    if (!animate) return;
    let alive = true;
    const timers: ReturnType<typeof setTimeout>[] = [];
    const run = () => {
      step = 0;
      typed = 0;
      let i = 0;
      const advance = () => {
        if (!alive) return;
        if (i >= lines.length) {
          timers.push(
            setTimeout(() => {
              if (alive) run();
            }, 3600),
          );
          return;
        }
        const line = lines[i];
        if (line.t === "cmd") {
          let c = 0;
          const typeChar = () => {
            if (!alive) return;
            c++;
            typed = c;
            if (c < line.text.length) {
              timers.push(setTimeout(typeChar, 24 + Math.random() * 30));
            } else {
              step = i + 1;
              i++;
              timers.push(setTimeout(advance, 480));
            }
          };
          step = i + 1;
          typed = 0;
          timers.push(setTimeout(typeChar, 280));
        } else {
          step = i + 1;
          i++;
          timers.push(setTimeout(advance, line.text === "" ? 90 : 240));
        }
      };
      timers.push(setTimeout(advance, 550));
    };
    run();
    return () => {
      alive = false;
      timers.forEach(clearTimeout);
    };
  });
</script>

<div class="rd-term">
  <div class="rd-term-bar">
    <span class="rd-term-dots">
      <span style="background:#e06c5acc"></span>
      <span style="background:#e0b84acc"></span>
      <span style="background:#5bb463cc"></span>
    </span>
    <span class="rd-term-title" style="color:{term.dim}">{title}</span>
  </div>
  <div class="rd-term-body">
    <!-- height reservation: full transcript, invisible, + a trailing blank line -->
    <div class="rd-term-reserve" aria-hidden="true">
      {#each lines as ln, i (i)}
        {#if ln.t === "cmd"}
          <div class="rd-term-line">
            <span class="rd-term-prompt">{cast.prompt} $</span>&nbsp;{ln.text}
          </div>
        {:else}
          <div class="rd-term-line">{ln.text || " "}</div>
        {/if}
      {/each}
      <div class="rd-term-line">&nbsp;</div>
    </div>
    <!-- animated overlay -->
    <div class="rd-term-overlay" aria-live="off">
      {#each lines as ln, i (i)}
        {#if i < reveal}
          {#if ln.t === "cmd"}
            {@const activeTyping = i === reveal - 1 && typed < ln.text.length}
            {@const shown =
              i === reveal - 1 ? ln.text.slice(0, typed) : ln.text}
            <div class="rd-term-line" style="color:{term.cmd}">
              <span class="rd-term-prompt" style="color: var(--accent)"
                >{cast.prompt} $</span
              >&nbsp;{shown}{#if activeTyping}<span
                  class="rd-term-caret"
                  style="background: var(--accent)"
                ></span>{/if}
            </div>
          {:else}
            <div class="rd-term-line" style="color:{lineColor(ln.t)}">
              {ln.text || " "}
            </div>
          {/if}
        {/if}
      {/each}
      {#if reveal >= lines.length}
        <div>
          <span class="rd-term-blockcaret" style="background: var(--accent)"
          ></span>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .rd-term {
    border-radius: 10px;
    overflow: hidden;
    border: 1px solid var(--colors-terminal-border);
    background: var(--colors-terminal-bg);
    box-shadow: 0 10px 30px -12px rgba(0, 0, 0, 0.5);
  }
  .rd-term-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 9px 13px;
    background: var(--colors-terminal-head);
    border-bottom: 1px solid var(--colors-terminal-border);
  }
  .rd-term-dots {
    display: flex;
    gap: 6px;
  }
  .rd-term-dots > span {
    width: 9px;
    height: 9px;
    border-radius: 999px;
  }
  .rd-term-title {
    font-family: "Geist Mono", ui-monospace, monospace;
    font-size: 11px;
    margin-left: 4px;
  }
  .rd-term-body {
    position: relative;
    font-family: "Geist Mono", ui-monospace, monospace;
    font-size: var(--font-sizes-caption);
    line-height: 1.7;
    padding: 14px 15px 16px;
  }
  .rd-term-reserve {
    visibility: hidden;
  }
  .rd-term-overlay {
    position: absolute;
    top: 14px;
    left: 15px;
    right: 15px;
  }
  .rd-term-line {
    white-space: pre-wrap;
    word-break: break-word;
  }
  .rd-term-prompt {
    font-weight: 600;
  }
  .rd-term-caret {
    display: inline-block;
    width: 7px;
    height: 14px;
    margin-left: 1px;
    transform: translateY(2px);
    animation: rd-term-blink 1s steps(1) infinite;
  }
  .rd-term-blockcaret {
    display: inline-block;
    width: 8px;
    height: 15px;
    opacity: 0.85;
    animation: rd-term-blink 1.1s steps(1) infinite;
  }
  @keyframes rd-term-blink {
    0%,
    50% {
      opacity: 1;
    }
    50.01%,
    100% {
      opacity: 0;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .rd-term-caret,
    .rd-term-blockcaret {
      animation: none;
    }
  }
</style>
