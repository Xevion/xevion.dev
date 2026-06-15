<script lang="ts">
  import { css, cx } from "styled-system/css";
  import type { TocItem } from "$lib/tiptap/render.server";

  let { toc }: { toc: TocItem[] } = $props();

  // Active heading drives the scroll-spy highlight.
  let activeId = $state("");

  // Track which headings sit inside a band near the top of the viewport; the
  // active one is the topmost of those by document order. When the band is empty
  // (scrolled between headings), keep the last active rather than clearing.
  $effect(() => {
    // Seed with the first heading so the rail isn't blank before the first
    // observer callback fires.
    if (!activeId) activeId = toc[0]?.id ?? "";
    const els = toc
      .map((item) => document.getElementById(item.id))
      .filter((el): el is HTMLElement => el !== null);
    if (els.length === 0) return;

    const order = toc.map((item) => item.id);
    let visible: string[] = [];
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            if (!visible.includes(entry.target.id))
              visible.push(entry.target.id);
          } else {
            visible = visible.filter((id) => id !== entry.target.id);
          }
        }
        const top = order.find((id) => visible.includes(id));
        if (top) activeId = top;
      },
      { rootMargin: "-80px 0px -65% 0px", threshold: 0 },
    );
    els.forEach((el) => observer.observe(el));
    return () => observer.disconnect();
  });

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

<nav class="rd-toc" aria-label="On this page">
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
