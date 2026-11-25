<script lang="ts">
  import { SvelteURLSearchParams } from "svelte/reactivity";
  import { css, cx } from "styled-system/css";
  import { hstack, center, wrap, grid } from "styled-system/patterns";
  import type { IconCollection, IconSearchResult } from "#lib/types/icons.ts";
  import { parseIconIdentifier } from "#lib/types/icons.ts";
  import Icon from "#lib/components/Icon.svelte";
  import { getLogger } from "@logtape/logtape";
  import {
    labelClass,
    helpTextClass,
    adminInputBase,
    dropdownPanelClass,
  } from "#lib/styles/admin.ts";

  const logger = getLogger(["admin", "components", "IconPicker"]);

  const ICONIFY = "https://api.iconify.design";
  const RESULT_LIMIT = 100;
  const COLLECTION_TABS = [
    "lucide",
    "simple-icons",
    "devicon",
    "logos",
    "material-symbols",
    "tabler",
    "heroicons",
    "feather",
  ];

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
  let searchResults = $state<IconSearchResult[]>([]);
  let collections = $state<IconCollection[]>([]);
  let selectedCollection = $state<string>("all");
  let isLoading = $state(false);
  let showDropdown = $state(false);
  let searchSeq = 0;

  const inputId = `iconpicker-${Math.random().toString(36).substring(2, 11)}`;

  // Results preview straight from Iconify so browsing never writes to our own bucket.
  const previewStyle = (collection: string, name: string) => {
    const url = `${ICONIFY}/${collection}/${name}.svg`;
    return `background-color: currentColor; mask: url('${url}') center / contain no-repeat; -webkit-mask: url('${url}') center / contain no-repeat;`;
  };

  $effect(() => {
    void loadCollections();
  });

  $effect(() => {
    const query = searchQuery;
    const collection = selectedCollection;
    if (!query) {
      searchResults = [];
      showDropdown = false;
      return undefined;
    }
    const timer = setTimeout(() => void performSearch(query, collection), 300);
    return () => clearTimeout(timer);
  });

  async function loadCollections() {
    try {
      const response = await fetch(
        `${ICONIFY}/collections?prefixes=${COLLECTION_TABS.join(",")}`,
      );
      if (!response.ok) return;
      const data = (await response.json()) as Record<
        string,
        { name: string; total: number; category?: string }
      >;
      collections = COLLECTION_TABS.flatMap((id) =>
        data[id] ? [{ id, ...data[id] }] : [],
      );
    } catch (error) {
      logger.error("Failed to load collections", { error });
    }
  }

  function toResult(identifier: string): IconSearchResult[] {
    const parsed = parseIconIdentifier(identifier);
    return parsed
      ? [
          {
            identifier: identifier as IconSearchResult["identifier"],
            ...parsed,
          },
        ]
      : [];
  }

  async function fetchIcons(prefix: string | null, term: string) {
    if (prefix && !term) {
      const response = await fetch(`${ICONIFY}/collection?prefix=${prefix}`);
      if (!response.ok) return [];
      const data = (await response.json()) as {
        uncategorized?: string[];
        categories?: Record<string, string[]>;
      };
      const names = new Set([
        ...(data.uncategorized ?? []),
        ...Object.values(data.categories ?? {}).flat(),
      ]);
      return [...names]
        .slice(0, RESULT_LIMIT)
        .flatMap((name) => toResult(`${prefix}:${name}`));
    }
    const params = new SvelteURLSearchParams({
      query: term,
      limit: String(RESULT_LIMIT),
    });
    if (prefix) params.set("prefix", prefix);
    const response = await fetch(`${ICONIFY}/search?${params}`);
    if (!response.ok) return [];
    const data = (await response.json()) as { icons?: string[] };
    return (data.icons ?? []).flatMap(toResult);
  }

  async function performSearch(query: string, collection: string) {
    const seq = ++searchSeq;
    isLoading = true;
    showDropdown = true;

    // "lucide:home" narrows to one collection regardless of the active tab.
    const colon = query.indexOf(":");
    const prefix =
      colon === -1
        ? collection === "all"
          ? null
          : collection
        : query.slice(0, colon);
    const term = (colon === -1 ? query : query.slice(colon + 1)).trim();

    try {
      const results = await fetchIcons(prefix, term);
      if (seq === searchSeq) searchResults = results;
    } catch (error) {
      logger.error("Failed to search icons", { error });
    } finally {
      if (seq === searchSeq) isLoading = false;
    }
  }

  function selectIcon(identifier: string) {
    selectedIcon = identifier;
    searchQuery = "";
    showDropdown = false;
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
      <div class={center({ w: "10", h: "10", rounded: "sm", bg: "admin.bg" })}>
        <Icon icon={selectedIcon} sizeClass={css({ w: "6", h: "6" })} />
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
            <button
              type="button"
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
              <span
                class={css({ w: "9", h: "9", color: "admin.text" })}
                style={previewStyle(result.collection, result.name)}
                role="img"
                aria-label={result.identifier}
              ></span>

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
