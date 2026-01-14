<script lang="ts">
  import { cn } from "$lib/utils";
  import TagChip from "$lib/components/TagChip.svelte";
  import type { TagWithIcon } from "$lib/admin-types";

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

<div class={cn("space-y-1.5", className)}>
  {#if label}
    <label for={inputId} class="block text-sm font-medium text-admin-text">
      {label}
    </label>
  {/if}

  <div class="relative">
    <!-- Selected tags display -->
    <div
      class="min-h-[42px] w-full rounded-md border border-admin-border bg-admin-bg-secondary px-3 py-2"
    >
      <div class="flex flex-wrap gap-1.5 items-center">
        {#each selectedTags as tag (tag.id)}
          <button
            type="button"
            onclick={() => removeTag(tag.id)}
            onmouseenter={() => (hoveredTagId = tag.id)}
            onmouseleave={() => (hoveredTagId = null)}
            class="cursor-pointer"
            aria-label="Remove {tag.name}"
          >
            <TagChip
              name={tag.name}
              color={hoveredTagId === tag.id ? "ef4444" : tag.color}
              iconSvg={tag.iconSvg}
              class="transition-all duration-150 {hoveredTagId === tag.id
                ? 'bg-red-100/80 dark:bg-red-900/40'
                : ''}"
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
          class="flex-1 bg-transparent text-sm text-admin-text placeholder:text-admin-text-muted focus:outline-none min-w-[120px]"
          onfocus={handleInputFocus}
          onblur={handleInputBlur}
        />
      </div>
    </div>

    <!-- Dropdown -->
    {#if dropdownOpen && filteredTags.length > 0}
      <div
        class="absolute z-10 mt-1 max-h-60 w-full overflow-auto rounded-md border border-admin-border bg-admin-surface py-1 shadow-lg"
      >
        {#each filteredTags as tag (tag.id)}
          <button
            type="button"
            class="w-full px-3 py-1.5 text-left hover:bg-admin-surface-hover transition-colors flex items-center"
            onclick={() => addTag(tag.id)}
          >
            <TagChip name={tag.name} color={tag.color} iconSvg={tag.iconSvg} />
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <p class="text-xs text-admin-text-muted">
    {selectedTagIds.length} tag{selectedTagIds.length === 1 ? "" : "s"} selected
  </p>
</div>
