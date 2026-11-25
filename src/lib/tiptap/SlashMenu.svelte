<script lang="ts">
  import { css, cx } from "styled-system/css";
  import type { SlashMenuState } from "./slash-command.svelte";

  let { menu }: { menu: SlashMenuState } = $props();

  let listEl = $state<HTMLDivElement>();

  // Keep the keyboard-selected row in view as the user arrows through a list
  // longer than the popup's max height.
  $effect(() => {
    const active = listEl?.querySelector<HTMLElement>('[aria-selected="true"]');
    active?.scrollIntoView({ block: "nearest" });
  });

  const menuClass = css({
    display: "flex",
    flexDirection: "column",
    gap: "0.5",
    minW: "13rem",
    maxH: "18rem",
    overflowY: "auto",
    p: "1",
    rounded: "md",
    borderWidth: "1px",
    borderColor: "admin.border",
    bg: "admin.surface",
    shadow: "lg",
    fontSize: "sm",
  });

  const itemClass = css({
    textAlign: "left",
    px: "2",
    py: "1.5",
    rounded: "sm",
    color: "admin.text",
    cursor: "pointer",
    transition: "colors",
    _hover: { bg: "admin.surfaceHover" },
  });

  const itemActiveClass = css({
    bg: "admin.accent/15",
    color: "admin.accent",
  });

  const emptyClass = css({
    px: "2",
    py: "1.5",
    color: "admin.textSecondary",
  });
</script>

<div
  bind:this={listEl}
  class={menuClass}
  role="listbox"
  aria-label="Insert block"
>
  {#if menu.items.length === 0}
    <div class={emptyClass}>No matching blocks</div>
  {:else}
    {#each menu.items as item, i (item.title)}
      <button
        type="button"
        role="option"
        aria-selected={i === menu.selectedIndex}
        class={cx(itemClass, i === menu.selectedIndex ? itemActiveClass : "")}
        onmousedown={(e) => {
          // Keep editor focus/selection intact until the command runs.
          e.preventDefault();
          menu.select(item);
        }}
        onmouseenter={() => (menu.selectedIndex = i)}
      >
        {item.title}
      </button>
    {/each}
  {/if}
</div>
