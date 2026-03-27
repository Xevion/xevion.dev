<script lang="ts">
  import { css, cx } from "styled-system/css";
  import { hstack, center, wrap, grid } from "styled-system/patterns";
  import { SvelteMap } from "svelte/reactivity";
  import type { IconCollection } from "$lib/types/icons";
  import { getLogger } from "@logtape/logtape";
  import {
    labelClass,
    helpTextClass,
    adminInputBase,
    dropdownPanelClass,
  } from "$lib/styles/admin";

  const logger = getLogger(["admin", "components", "IconPicker"]);

  interface Props {
    selectedIcon: string;
    label?: string;
    placeholder?: string;
    class?: string;
  }

  let {
    selectedIcon = $bindable(""),
    label,
    placeholder = "Search icons... (e.g., lucide:home or just home)",
    class: className,
  }: Props = $props();

  let searchQuery = $state("");
  let searchResults = $state<
    Array<{ identifier: string; collection: string; name: string }>
  >([]);
  let collections = $state<IconCollection[]>([]);
  let selectedCollection = $state<string>("all");
  let isLoading = $state(false);
  let showDropdown = $state(false);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  // Load SVG cache for preview
  let iconSvgCache = new SvelteMap<string, string>();
  let selectedIconSvg = $state<string | null>(null);

  // IntersectionObserver for lazy loading icons
  let observer: IntersectionObserver | null = null;

  // Generate unique ID for accessibility
  const inputId = `iconpicker-${Math.random().toString(36).substring(2, 11)}`;

  // Load collections on mount and setup observer
  $effect(() => {
    loadCollections();
    setupIntersectionObserver();

    return () => {
      if (observer) {
        observer.disconnect();
      }
    };
  });

  // Load selected icon SVG
  $effect(() => {
    if (selectedIcon) {
      loadIconSvg(selectedIcon);
    }
  });

  function setupIntersectionObserver() {
    observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            const identifier = entry.target.getAttribute("data-icon-id");
            if (identifier && !iconSvgCache.has(identifier)) {
              loadIconSvg(identifier);
            }
          }
        }
      },
      {
        root: null,
        rootMargin: "50px",
        threshold: 0.01,
      },
    );
  }

  // Debounced search
  $effect(() => {
    if (searchQuery) {
      if (debounceTimer) {
        clearTimeout(debounceTimer);
      }

      debounceTimer = setTimeout(() => {
        performSearch();
      }, 300);
    } else {
      searchResults = [];
      showDropdown = false;
    }
  });

  async function loadCollections() {
    try {
      const response = await fetch("/api/icons/collections");
      if (response.ok) {
        const data = await response.json();
        collections = data.collections;
      }
    } catch (error) {
      logger.error("Failed to load collections", { error });
    }
  }

  async function performSearch() {
    isLoading = true;
    showDropdown = true;

    try {
      // Build query with collection filter if not "all"
      let query = searchQuery;
      if (selectedCollection !== "all" && !query.includes(":")) {
        query = `${selectedCollection}:${query}`;
      }

      const response = await fetch(
        `/api/icons/search?q=${encodeURIComponent(query)}&limit=100`,
      );
      if (response.ok) {
        const data = await response.json();
        searchResults = data.icons;

        // Wait for DOM to update, then observe icon elements
        setTimeout(() => observeIconElements(), 100);
      }
    } catch (error) {
      logger.error("Failed to search icons", { error });
    } finally {
      isLoading = false;
    }
  }

  function observeIconElements() {
    if (!observer) return;

    // Find all icon button elements and observe them
    const iconButtons = document.querySelectorAll(`[data-icon-id]`);
    for (const button of iconButtons) {
      observer.observe(button);
    }
  }

  async function loadIconSvg(identifier: string) {
    // Check cache first
    if (iconSvgCache.has(identifier)) {
      if (identifier === selectedIcon) {
        selectedIconSvg = iconSvgCache.get(identifier)!;
      }
      return;
    }

    try {
      const [collection, name] = identifier.split(":");
      const response = await fetch(`/api/icons/${collection}/${name}`);
      if (response.ok) {
        const data = await response.json();
        iconSvgCache.set(identifier, data.svg);
        if (identifier === selectedIcon) {
          selectedIconSvg = data.svg;
        }
      }
    } catch (error) {
      logger.error("Failed to load icon SVG", { error, identifier });
      if (identifier === selectedIcon) {
        selectedIconSvg = null;
      }
    }
  }

  function selectIcon(identifier: string) {
    selectedIcon = identifier;
    searchQuery = "";
    showDropdown = false;
    loadIconSvg(identifier);
  }

  function handleInputFocus() {
    if (searchQuery && searchResults.length > 0) {
      showDropdown = true;
    }
  }

  function handleInputBlur() {
    setTimeout(() => {
      showDropdown = false;
    }, 200);
  }

  function clearSelection() {
    selectedIcon = "";
    selectedIconSvg = null;
  }
</script>

<div class={cx(css({ spaceY: "2" }), className)}>
  {#if label}
    <label for={inputId} class={labelClass}>
      {label}
    </label>
  {/if}

  <!-- Selected icon preview -->
  {#if selectedIcon}
    <div
      class={hstack({
        gap: "3",
        rounded: "md",
        borderWidth: "1px",
        borderColor: "admin.border",
        bg: "admin.bgSecondary",
        p: "3",
      })}
    >
      <div
        class={center({ w: "10", h: "10", rounded: "sm", bg: "admin.bg" })}
        data-icon-container
      >
        {#if selectedIconSvg}
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          {@html selectedIconSvg}
        {:else}
          <div
            class={css({
              w: "6",
              h: "6",
              animation: "pulse",
              rounded: "sm",
              bg: "admin.surfaceHover",
            })}
          ></div>
        {/if}
      </div>
      <div class={css({ flex: "1" })}>
        <p
          class={css({
            fontSize: "sm",
            fontWeight: "medium",
            color: "admin.text",
          })}
        >
          {selectedIcon}
        </p>
      </div>
      <button
        type="button"
        onclick={clearSelection}
        class={css({
          rounded: "sm",
          px: "2",
          py: "1",
          fontSize: "sm",
          color: "admin.textMuted",
          _hover: { bg: "admin.surfaceHover", color: "admin.text" },
        })}
      >
        Clear
      </button>
    </div>
  {/if}

  <!-- Collection tabs -->
  <div class={wrap({ gap: "1" })}>
    <button
      type="button"
      class={cx(
        css({
          rounded: "md",
          px: "3",
          py: "1.5",
          fontSize: "sm",
          fontWeight: "medium",
          transition: "colors",
        }),
        selectedCollection === "all"
          ? css({ bg: "admin.accent", color: "white" })
          : css({
              bg: "admin.surface",
              color: "admin.textMuted",
              _hover: { bg: "admin.surfaceHover", color: "admin.text" },
            }),
      )}
      onclick={() => (selectedCollection = "all")}
    >
      All
    </button>
    {#each collections as collection (collection.id)}
      <button
        type="button"
        class={cx(
          css({
            rounded: "md",
            px: "3",
            py: "1.5",
            fontSize: "sm",
            fontWeight: "medium",
            transition: "colors",
          }),
          selectedCollection === collection.id
            ? css({ bg: "admin.accent", color: "white" })
            : css({
                bg: "admin.surface",
                color: "admin.textMuted",
                _hover: { bg: "admin.surfaceHover", color: "admin.text" },
              }),
        )}
        onclick={() => (selectedCollection = collection.id)}
      >
        {collection.name}
        <span class={css({ ml: "1", fontSize: "xs", opacity: "0.6" })}
          >({collection.total})</span
        >
      </button>
    {/each}
  </div>

  <!-- Search input -->
  <div class={css({ position: "relative" })}>
    <input
      id={inputId}
      type="text"
      bind:value={searchQuery}
      {placeholder}
      class={adminInputBase}
      onfocus={handleInputFocus}
      onblur={handleInputBlur}
    />

    <!-- Search results dropdown -->
    {#if showDropdown && searchResults.length > 0}
      <div class={cx(dropdownPanelClass, css({ maxH: "96" }))}>
        <!-- Grid layout for icons -->
        <div class={grid({ columns: 8, gap: "1", p: "2" })}>
          {#each searchResults as result (result.identifier)}
            {@const cachedSvg = iconSvgCache.get(result.identifier)}
            <button
              type="button"
              data-icon-id={result.identifier}
              class={cx(
                "group",
                center({
                  position: "relative",
                  w: "12",
                  h: "12",
                  rounded: "sm",
                  _hover: { bg: "admin.surfaceHover" },
                }),
              )}
              onclick={() => selectIcon(result.identifier)}
              title={result.identifier}
            >
              <!-- Lazy load icon SVG via IntersectionObserver -->
              <div
                class={css({ w: "9", h: "9", color: "admin.text" })}
                data-icon-container
              >
                {#if cachedSvg}
                  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                  {@html cachedSvg}
                {:else}
                  <div
                    class={css({
                      w: "full",
                      h: "full",
                      animation: "pulse",
                      rounded: "sm",
                      bg: "admin.surfaceHover",
                    })}
                  ></div>
                {/if}
              </div>

              <!-- Tooltip on hover -->
              <div
                class={css({
                  pointerEvents: "none",
                  position: "absolute",
                  top: "-8",
                  left: "50%",
                  zIndex: 20,
                  display: "none",
                  transform: "translateX(-50%)",
                  whiteSpace: "nowrap",
                  rounded: "sm",
                  bg: "admin.surface",
                  borderWidth: "1px",
                  borderColor: "admin.border",
                  px: "2",
                  py: "1",
                  fontSize: "xs",
                  color: "admin.text",
                  _groupHover: { display: "block" },
                })}
              >
                {result.name}
              </div>
            </button>
          {/each}
        </div>

        {#if isLoading}
          <div
            class={css({
              borderTopWidth: "1px",
              borderColor: "admin.border",
              p: "3",
              textAlign: "center",
              fontSize: "sm",
              color: "admin.textMuted",
            })}
          >
            Loading...
          </div>
        {/if}
      </div>
    {:else if showDropdown && searchQuery && !isLoading}
      <div
        class={cx(
          dropdownPanelClass,
          css({
            p: "3",
            textAlign: "center",
            fontSize: "sm",
            color: "admin.textMuted",
          }),
        )}
      >
        No icons found for "{searchQuery}"
      </div>
    {/if}
  </div>

  <p class={helpTextClass}>
    Tip: Use "collection:search" to filter (e.g., "lucide:home" or
    "simple-icons:react")
  </p>
</div>

<!-- TODO: Future enhancement - Recent/favorite icons -->
<!-- Store recently used icons in localStorage for quick access -->
<!-- Could add "star" button to favorite frequently used icons -->

<style>
  /* Ensure dynamically-injected SVG icons fill their container */
  [data-icon-container] :global(svg) {
    width: 100%;
    height: 100%;
  }
</style>
