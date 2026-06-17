import type { TocItem } from "$lib/tiptap/render.server";

/**
 * Scroll-spy for the table of contents using a "playhead" reading line.
 *
 * The line, in document coordinates, sits at `progress · scrollHeight` — it
 * scrubs the whole document height in lockstep with the scrollbar. The active
 * heading is the last one whose top is at or above that line; equivalently, a
 * heading lights up once scroll progress passes its fractional position in the
 * document.
 *
 * Why not a fixed band/line: a stationary line near the top can never be reached
 * by headings in the final viewport-height of the document (the page bottoms out
 * first), so trailing sections never highlight. A line that reaches the document
 * bottom at full scroll makes every section — including short trailing ones —
 * reachable by construction. The cost is that the highlight "runs ahead" a little
 * near the end; that drift is the mechanism that makes the tail reachable.
 *
 * The page scrolls on the document root (OverlayScrollbars keeps native
 * `html { overflow: scroll }` and only restyles the scrollbar), so window scroll
 * metrics are authoritative; `headingTop` from getBoundingClientRect is already
 * relative to the viewport top, which the line is measured from.
 *
 * Returns a reactive `activeId`. Call during component init so the internal
 * `$effect` binds to that component's lifecycle.
 */
export function createTocSpy(toc: () => TocItem[]) {
  let activeId = $state("");

  $effect(() => {
    const items = toc();
    if (items.length === 0) return;

    // Seed with the first heading: this is also the top-of-page clamp. When the
    // playhead is above every heading (reading content before the first one),
    // nothing satisfies the test and `current` stays on the first item.
    activeId ||= items[0].id;

    let frame = 0;
    const compute = () => {
      frame = 0;
      const clientHeight = document.documentElement.clientHeight;
      const maxScroll = document.documentElement.scrollHeight - clientHeight;

      // Non-scrollable (content fits the viewport): first heading wins.
      if (maxScroll <= 0) {
        activeId = items[0].id;
        return;
      }

      const progress = Math.min(1, Math.max(0, window.scrollY / maxScroll));
      const line = progress * clientHeight;

      // Headings are in document order, so the last one above the line is the
      // active section; once one sits below it, every later one does too.
      let current = items[0].id;
      for (const item of items) {
        const el = document.getElementById(item.id);
        if (!el) continue;
        if (el.getBoundingClientRect().top > line) break;
        current = item.id;
      }
      activeId = current;
    };

    // Coalesce scroll/resize bursts to one measurement per frame.
    const schedule = () => {
      if (!frame) frame = requestAnimationFrame(compute);
    };

    compute();
    window.addEventListener("scroll", schedule, { passive: true });
    window.addEventListener("resize", schedule, { passive: true });
    return () => {
      if (frame) cancelAnimationFrame(frame);
      window.removeEventListener("scroll", schedule);
      window.removeEventListener("resize", schedule);
    };
  });

  return {
    get activeId() {
      return activeId;
    },
  };
}
