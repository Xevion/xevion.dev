<script lang="ts">
  import { cn } from "$lib/utils";
  import type { AdminTag } from "$lib/admin-types";
  import IconX from "~icons/lucide/x";

  interface Props {
    label?: string;
    availableTags: AdminTag[];
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

  const selectedTags = $derived(
    availableTags.filter((tag) => selectedTagIds.includes(tag.id))
  );

  const filteredTags = $derived(
    availableTags.filter(
      (tag) =>
        !selectedTagIds.includes(tag.id) &&
        tag.name.toLowerCase().includes(searchTerm.toLowerCase())
    )
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
    <label class="block text-sm font-medium text-admin-text">
      {label}
    </label>
  {/if}

  <div class="relative">
    <!-- Selected tags display -->
    <div
      class="min-h-[42px] w-full rounded-md border border-admin-border bg-admin-panel px-3 py-2"
    >
      <div class="flex flex-wrap gap-2">
        {#each selectedTags as tag}
          <span
            class="inline-flex items-center gap-1 rounded-full bg-blue-500/10 px-2.5 py-0.5 text-xs font-medium text-blue-400 ring-1 ring-inset ring-blue-500/20"
          >
            {tag.name}
            <button
              type="button"
              onclick={() => removeTag(tag.id)}
              class="hover:text-blue-300"
              aria-label="Remove tag"
            >
              <IconX class="w-3 h-3" />
            </button>
          </span>
        {/each}

        <!-- Search input -->
        <input
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
        class="absolute z-10 mt-1 max-h-60 w-full overflow-auto rounded-md border border-admin-border bg-admin-panel py-1 shadow-lg"
      >
        {#each filteredTags as tag}
          <button
            type="button"
            class="w-full px-3 py-2 text-left text-sm text-admin-text hover:bg-admin-hover transition-colors"
            onclick={() => addTag(tag.id)}
          >
            {tag.name}
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <p class="text-xs text-admin-text-muted">
    {selectedTagIds.length} tag{selectedTagIds.length === 1 ? "" : "s"} selected
  </p>
</div>
