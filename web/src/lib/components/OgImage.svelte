<script lang="ts">
  import type { OGImageSpec } from "$lib/og-types";

  type Props = {
    title: string;
    subtitle?: string;
    type: OGImageSpec["type"];
  };

  let { title, subtitle, type }: Props = $props();

  // Calculate font size based on title length (matching original logic)
  const fontSize = $derived(title.length > 40 ? "60px" : "72px");

  // OG images are always dark-themed. Satori can't resolve CSS variables,
  // so we use raw hex values that match the design token palette.
  const og = {
    bg: "#000000", // bg.primary (dark)
    text: "#fafafa", // text.primary (dark) — zinc.50
    heading: "#ffffff", // white
    subtitle: "#a1a1aa", // text.tertiary (dark) — zinc.400
    border: "#27272a", // border.DEFAULT (dark) — zinc.800
    muted: "#71717a", // zinc.500
    dimmed: "#52525b", // text.secondary (base) — zinc.600
    fontBody: "'Schibsted Grotesk', sans-serif",
    fontHeading: "'Hanken Grotesk', sans-serif",
  } as const;
</script>

<div
  style="display: flex; width: 1200px; height: 630px; background-color: {og.bg}; color: {og.text}; font-family: {og.fontBody}; padding: 60px 80px;"
>
  <div
    style="display: flex; flex-direction: column; justify-content: space-between; width: 100%; height: 100%;"
  >
    <!-- Content section -->
    <div
      style="display: flex; flex-direction: column; flex: 1; justify-content: center;"
    >
      <h1
        style="font-family: {og.fontHeading}; font-weight: 900; font-size: {fontSize}; line-height: 1.1; margin: 0; color: {og.heading};"
      >
        {title}
      </h1>
      {#if subtitle}
        <p
          style="font-family: {og.fontBody}; font-size: 36px; margin: 32px 0 0 0; color: {og.subtitle}; line-height: 1.4;"
        >
          {subtitle}
        </p>
      {/if}
    </div>

    <!-- Footer -->
    <div
      style="display: flex; justify-content: space-between; align-items: flex-end; border-top: 2px solid {og.border}; padding-top: 24px;"
    >
      <div style="font-size: 28px; color: {og.muted}; font-weight: 500;">
        xevion.dev
      </div>
      {#if type === "project"}
        <div
          style="font-size: 24px; color: {og.dimmed}; text-transform: uppercase; letter-spacing: 0.05em;"
        >
          PROJECT
        </div>
      {/if}
    </div>
  </div>
</div>
