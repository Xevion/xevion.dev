<script lang="ts">
  import { cn } from "$lib/utils";

  interface Props {
    selectedColor: string | undefined;
    label?: string;
    class?: string;
  }

  let {
    selectedColor = $bindable(),
    label = "Color",
    class: className,
  }: Props = $props();

  // Preset color palette (Tailwind-inspired)
  const PRESET_COLORS = [
    { name: "Cyan", value: "06b6d4" },
    { name: "Blue", value: "3b82f6" },
    { name: "Indigo", value: "6366f1" },
    { name: "Purple", value: "a855f7" },
    { name: "Pink", value: "ec4899" },
    { name: "Rose", value: "f43f5e" },
    { name: "Orange", value: "f97316" },
    { name: "Amber", value: "f59e0b" },
    { name: "Yellow", value: "eab308" },
    { name: "Lime", value: "84cc16" },
    { name: "Green", value: "22c55e" },
    { name: "Emerald", value: "10b981" },
    { name: "Teal", value: "14b8a6" },
    { name: "Sky", value: "0ea5e9" },
    { name: "Zinc", value: "a1a1aa" },
  ];

  let customHex = $state(selectedColor || "");
  let validationError = $state<string | null>(null);

  // Validate hex format (6 characters, no hash, no alpha)
  function validateHexColor(hex: string): boolean {
    return /^[0-9a-fA-F]{6}$/.test(hex);
  }

  function handleCustomInput(event: Event) {
    const input = (event.target as HTMLInputElement).value.replace(
      /[^0-9a-fA-F]/g,
      "",
    );
    customHex = input.slice(0, 6);

    if (customHex.length === 6) {
      if (validateHexColor(customHex)) {
        selectedColor = customHex.toLowerCase();
        validationError = null;
      } else {
        validationError = "Invalid hex format";
      }
    } else if (customHex.length === 0) {
      selectedColor = undefined;
      validationError = null;
    } else {
      validationError = "Must be 6 characters";
    }
  }

  function selectPreset(hex: string) {
    selectedColor = hex;
    customHex = hex;
    validationError = null;
  }

  function clearColor() {
    selectedColor = undefined;
    customHex = "";
    validationError = null;
  }
</script>

<div class={cn("space-y-3", className)}>
  {#if label}
    <label class="block text-sm font-medium text-admin-text">{label}</label>
  {/if}

  <!-- Preset Palette -->
  <div class="grid grid-cols-8 gap-2">
    {#each PRESET_COLORS as preset (preset.value)}
      <button
        type="button"
        class={cn(
          "size-8 rounded border-2 transition-all hover:scale-110",
          selectedColor === preset.value
            ? "border-admin-accent ring-2 ring-admin-accent/20"
            : "border-admin-border hover:border-admin-border-hover",
        )}
        style="background-color: #{preset.value}"
        title={preset.name}
        onclick={() => selectPreset(preset.value)}
      />
    {/each}

    <!-- Clear button -->
    <button
      type="button"
      class={cn(
        "size-8 rounded border-2 transition-all hover:scale-110 flex items-center justify-center",
        !selectedColor
          ? "border-admin-accent ring-2 ring-admin-accent/20 bg-admin-surface-hover"
          : "border-admin-border hover:border-admin-border-hover bg-admin-surface",
      )}
      title="No color"
      onclick={clearColor}
    >
      <span class="text-admin-text-muted text-xs">✕</span>
    </button>
  </div>

  <!-- Custom Hex Input -->
  <div class="flex items-start gap-2">
    <div class="flex-1">
      <div class="relative">
        <span
          class="absolute left-3 top-1/2 -translate-y-1/2 text-admin-text-muted"
          >#</span
        >
        <input
          type="text"
          value={customHex}
          oninput={handleCustomInput}
          placeholder="3b82f6"
          maxlength="6"
          class={cn(
            "w-full rounded-md border bg-admin-bg-secondary px-3 py-2 pl-7 text-sm text-admin-text",
            "placeholder:text-admin-text-muted focus:outline-none focus:ring-2",
            validationError
              ? "border-red-500 focus:ring-red-500/20"
              : "border-admin-border focus:border-admin-border-hover focus:ring-admin-accent/20",
          )}
        />
      </div>
      {#if validationError}
        <p class="mt-1 text-xs text-red-400">{validationError}</p>
      {/if}
    </div>

    <!-- Color Preview -->
    {#if selectedColor && validateHexColor(selectedColor)}
      <div
        class="size-10 shrink-0 rounded-md border-2 border-admin-border"
        style="background-color: #{selectedColor}"
        title="#{selectedColor}"
      />
    {/if}
  </div>
</div>
