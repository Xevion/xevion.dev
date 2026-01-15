<script lang="ts">
  import { tick } from "svelte";
  import { themeStore } from "$lib/stores/theme.svelte";
  import { telemetry } from "$lib/telemetry";
  import IconSun from "~icons/lucide/sun";
  import IconMoon from "~icons/lucide/moon";

  /**
   * Theme toggle with View Transitions API circular reveal animation.
   * The clip-path circle expands from the click point to cover the viewport.
   */
  async function handleToggle(event: MouseEvent) {
    const newTheme = themeStore.isDark ? "light" : "dark";

    const supportsViewTransition =
      typeof document !== "undefined" &&
      "startViewTransition" in document &&
      !window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    if (!supportsViewTransition) {
      themeStore.toggle();
      telemetry.track({
        name: "theme_change",
        properties: { theme: newTheme },
      });
      return;
    }

    // Calculate animation origin from click coordinates
    const x = event.clientX;
    const y = event.clientY;
    const endRadius = Math.hypot(
      Math.max(x, innerWidth - x),
      Math.max(y, innerHeight - y),
    );

    // Remove view-transition-names so all elements are captured in root snapshot
    // (named elements would otherwise appear through "holes" in the circular reveal)
    const elementsWithVTN = document.querySelectorAll(
      '[style*="view-transition-name"]',
    );
    const savedStyles: Array<{ el: Element; style: string }> = [];
    elementsWithVTN.forEach((el) => {
      savedStyles.push({ el, style: el.getAttribute("style") || "" });
      (el as HTMLElement).style.viewTransitionName = "none";
    });
    void document.documentElement.offsetHeight;

    const transition = document.startViewTransition(async () => {
      themeStore.toggle();
      await tick();
    });

    transition.ready.then(() => {
      document.documentElement.animate(
        {
          clipPath: [
            `circle(0px at ${x}px ${y}px)`,
            `circle(${endRadius}px at ${x}px ${y}px)`,
          ],
        },
        {
          duration: 500,
          easing: "cubic-bezier(0.4, 0, 0.2, 1)",
          pseudoElement: "::view-transition-new(root)",
        },
      );
    });

    await transition.finished;

    // Restore original view-transition-name styles
    savedStyles.forEach(({ el, style }) => {
      el.setAttribute("style", style);
    });

    telemetry.track({ name: "theme_change", properties: { theme: newTheme } });
  }
</script>

<button
  type="button"
  onclick={(e) => handleToggle(e)}
  aria-label={themeStore.isDark
    ? "Switch to light mode"
    : "Switch to dark mode"}
  class="relative size-9 rounded-md border border-zinc-300 dark:border-zinc-700 bg-zinc-100 dark:bg-zinc-900/50 hover:bg-zinc-200 dark:hover:bg-zinc-800/70 transition-all duration-200 cursor-pointer"
>
  <div class="absolute inset-0 flex items-center justify-center">
    <IconSun
      class="size-5 text-zinc-600 dark:text-zinc-400 transition-all duration-300 {themeStore.isDark
        ? 'rotate-90 scale-0 opacity-0'
        : 'rotate-0 scale-100 opacity-100'}"
    />
    <IconMoon
      class="absolute size-5 text-zinc-600 dark:text-zinc-400 transition-all duration-300 {themeStore.isDark
        ? 'rotate-0 scale-100 opacity-100'
        : '-rotate-90 scale-0 opacity-0'}"
    />
  </div>
</button>
