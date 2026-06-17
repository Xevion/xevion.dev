<script lang="ts">
  import { css, cx } from "styled-system/css";
  import type { TocItem } from "$lib/tiptap/render.server";

  // Presentational: the active heading is computed by the page-level scroll-spy
  // (see toc-spy.svelte.ts) and passed in, so the desktop rail and the mobile
  // overlay share one source of truth and one scroll listener.
  let { toc, activeId }: { toc: TocItem[]; activeId: string } = $props();

  const link = css({
    display: "block",
    py: "3px",
    fontSize: "13px",
    lineHeight: "1.4",
    color: "zinc.500",
    textDecoration: "none",
    borderLeftWidth: "2px",
    borderColor: "transparent",
    transition: "color .12s, border-color .12s",
    _hover: { color: "zinc.800", _dark: { color: "zinc.200" } },
  });
  const active = css({
    color: "var(--accent)",
    borderColor: "var(--accent)",
    fontWeight: "600",
    _hover: { color: "var(--accent)" },
  });
</script>

<nav
  class={css({ "@media (max-width: 760px)": { display: "none" } })}
  aria-label="On this page"
>
  <span class={css({ textStyle: "label.micro" })}>On this page</span>
  <ul
    class={css({
      mt: "10px",
      display: "flex",
      flexDirection: "column",
      listStyle: "none",
      p: "0",
      m: "0",
    })}
  >
    {#each toc as item (item.id)}
      <li>
        <a
          href="#{item.id}"
          class={cx(link, item.id === activeId && active)}
          style="padding-left: {item.level === 2 ? 10 : 22}px"
          aria-current={item.id === activeId ? "true" : undefined}
        >
          {item.text}
        </a>
      </li>
    {/each}
  </ul>
</nav>
