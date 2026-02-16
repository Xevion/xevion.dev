<script lang="ts">
  import { tick } from "svelte";
  import { css, cx } from "styled-system/css";
  import { center } from "styled-system/patterns";
  import { themeStore } from "$lib/stores/theme.svelte";
  import { telemetry } from "$lib/telemetry";
  import { iconButton } from "styled-system/recipes";
  import { iconMd } from "$lib/styles/admin";
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

  const themeIconBase = css({
    color: "zinc.600",
    transition: "all",
    transitionDuration: "300ms",
    _dark: { color: "zinc.400" },
  });
</script>

<button
  type="button"
  onclick={(e) => handleToggle(e)}
  aria-label={themeStore.isDark
    ? "Switch to light mode"
    : "Switch to dark mode"}
  class={iconButton()}
>
  <div class={center({ position: "absolute", inset: "0" })}>
    <IconSun
      class={cx(
        iconMd,
        themeIconBase,
        themeStore.isDark
          ? css({ rotate: "90deg", scale: "0", opacity: "0" })
          : css({ rotate: "0deg", scale: "1", opacity: "1" }),
      )}
    />
    <IconMoon
      class={cx(
        iconMd,
        css({ position: "absolute" }),
        themeIconBase,
        themeStore.isDark
          ? css({ rotate: "0deg", scale: "1", opacity: "1" })
          : css({ rotate: "-90deg", scale: "0", opacity: "0" }),
      )}
    />
  </div>
</button>
