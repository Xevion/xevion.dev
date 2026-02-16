<script lang="ts">
  import { css, cx } from "styled-system/css";
  import { flex, wrap } from "styled-system/patterns";
  import TagChip from "$lib/components/TagChip.svelte";
  import type { TagWithIcon } from "$lib/admin-types";
  import {
    labelClass,
    helpTextClass,
    fieldWrapperClass,
    dropdownPanelClass,
  } from "$lib/styles/admin";

  interface Props {
    label?: string;
    availableTags: TagWithIcon[];
    selectedTagIds: string[];
    placeholder?: string;
    class?: string;
  }

  let {
    label,
    availableTags,
    selectedTagIds = $bindable([]),
    placeholder = "Search tags...",
    class: className,
  }: Props = $props();

  let searchTerm = $state("");
  let dropdownOpen = $state(false);
  let inputRef: HTMLInputElement | undefined = $state();
  let hoveredTagId = $state<string | null>(null);

  // Generate unique ID for accessibility
  const inputId = `tagpicker-${Math.random().toString(36).substring(2, 11)}`;

  const selectedTags = $derived(
    availableTags.filter((tag) => selectedTagIds.includes(tag.id)),
  );

  const filteredTags = $derived(
    availableTags.filter(
      (tag) =>
        !selectedTagIds.includes(tag.id) &&
        tag.name.toLowerCase().includes(searchTerm.toLowerCase()),
    ),
  );

  function addTag(tagId: string) {
    selectedTagIds = [...selectedTagIds, tagId];
    searchTerm = "";
    dropdownOpen = false;
    inputRef?.focus();
  }

  function removeTag(tagId: string) {
    selectedTagIds = selectedTagIds.filter((id) => id !== tagId);
  }

  function handleInputFocus() {
    dropdownOpen = true;
  }

  function handleInputBlur() {
    setTimeout(() => {
      dropdownOpen = false;
    }, 200);
  }
</script>

<div class={cx(fieldWrapperClass, className)}>
  {#if label}
    <label for={inputId} class={labelClass}>
      {label}
    </label>
  {/if}

  <div class={css({ position: "relative" })}>
    <!-- Selected tags display -->
    <div
      class={css({
        minH: "42px",
        w: "full",
        rounded: "md",
        borderWidth: "1px",
        borderColor: "admin.border",
        bg: "admin.bgSecondary",
        px: "3",
        py: "2",
      })}
    >
      <div class={wrap({ gap: "1.5", align: "center" })}>
        {#each selectedTags as tag (tag.id)}
          <button
            type="button"
            onclick={() => removeTag(tag.id)}
            onmouseenter={() => (hoveredTagId = tag.id)}
            onmouseleave={() => (hoveredTagId = null)}
            class={css({ cursor: "pointer" })}
            aria-label="Remove {tag.name}"
          >
            <TagChip
              name={tag.name}
              color={hoveredTagId === tag.id ? "ef4444" : tag.color}
              icon={tag.icon}
              class={cx(
                css({ transition: "all", transitionDuration: "150ms" }),
                hoveredTagId === tag.id
                  ? css({ bg: { base: "red.100/80", _dark: "red.900/40" } })
                  : undefined,
              )}
            />
          </button>
        {/each}

        <!-- Search input -->
        <input
          id={inputId}
          bind:this={inputRef}
          type="text"
          bind:value={searchTerm}
          {placeholder}
          class={css({
            flex: "1",
            bg: "transparent",
            fontSize: "sm",
            color: "admin.text",
            _placeholder: { color: "admin.textMuted" },
            _focus: { outline: "none" },
            minW: "120px",
          })}
          onfocus={handleInputFocus}
          onblur={handleInputBlur}
        />
      </div>
    </div>

    <!-- Dropdown -->
    {#if dropdownOpen && filteredTags.length > 0}
      <div class={cx(dropdownPanelClass, css({ maxH: "60", py: "1" }))}>
        {#each filteredTags as tag (tag.id)}
          <button
            type="button"
            class={flex({
              align: "center",
              w: "full",
              px: "3",
              py: "1.5",
              textAlign: "left",
              _hover: { bg: "admin.surfaceHover" },
              transition: "colors",
            })}
            onclick={() => addTag(tag.id)}
          >
            <TagChip name={tag.name} color={tag.color} icon={tag.icon} />
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <p class={helpTextClass}>
    {selectedTagIds.length} tag{selectedTagIds.length === 1 ? "" : "s"} selected
  </p>
</div>
