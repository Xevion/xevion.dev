<script lang="ts">
  import { css, cx } from "styled-system/css";
  import { flex, center, grid } from "styled-system/patterns";
  import {
    labelClass,
    adminInputBase,
    adminInputError,
  } from "$lib/styles/admin";

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

<div class={cx(css({ spaceY: "3" }), className)}>
  {#if label}
    <div class={labelClass}>{label}</div>
  {/if}

  <!-- Preset Palette -->
  <div class={grid({ columns: 8, gap: "2" })}>
    {#each PRESET_COLORS as preset (preset.value)}
      <button
        type="button"
        class={cx(
          css({
            w: "8",
            h: "8",
            rounded: "sm",
            borderWidth: "2px",
            transition: "all",
            _hover: { transform: "scale(1.1)" },
          }),
          selectedColor === preset.value
            ? css({
                borderColor: "admin.accent",
                ringWidth: "2px",
                ringColor: "admin.accent/20",
              })
            : css({
                borderColor: "admin.border",
                _hover: { borderColor: "admin.borderHover" },
              }),
        )}
        style="background-color: #{preset.value}"
        title={preset.name}
        onclick={() => selectPreset(preset.value)}
      ></button>
    {/each}

    <!-- Clear button -->
    <button
      type="button"
      class={cx(
        center({
          w: "8",
          h: "8",
          rounded: "sm",
          borderWidth: "2px",
          transition: "all",
          _hover: { transform: "scale(1.1)" },
        }),
        !selectedColor
          ? css({
              borderColor: "admin.accent",
              ringWidth: "2px",
              ringColor: "admin.accent/20",
              bg: "admin.surfaceHover",
            })
          : css({
              borderColor: "admin.border",
              _hover: { borderColor: "admin.borderHover" },
              bg: "admin.surface",
            }),
      )}
      title="No color"
      onclick={clearColor}
    >
      <span class={css({ color: "admin.textMuted", fontSize: "xs" })}>✕</span>
    </button>
  </div>

  <!-- Custom Hex Input -->
  <div class={flex({ align: "flex-start", gap: "2" })}>
    <div class={css({ flex: "1" })}>
      <div class={css({ position: "relative" })}>
        <span
          class={css({
            position: "absolute",
            left: "3",
            top: "50%",
            transform: "translateY(-50%)",
            color: "admin.textMuted",
          })}>#</span
        >
        <input
          type="text"
          value={customHex}
          oninput={handleCustomInput}
          placeholder="3b82f6"
          maxlength="6"
          class={cx(
            adminInputBase,
            css({ pl: "7" }),
            validationError
              ? adminInputError
              : css({ _focus: { ringColor: "admin.accent/20" } }),
          )}
        />
      </div>
      {#if validationError}
        <p class={css({ mt: "1", fontSize: "xs", color: "red.400" })}>
          {validationError}
        </p>
      {/if}
    </div>

    <!-- Color Preview -->
    {#if selectedColor && validateHexColor(selectedColor)}
      <div
        class={css({
          w: "10",
          h: "10",
          flexShrink: "0",
          rounded: "md",
          borderWidth: "2px",
          borderColor: "admin.border",
        })}
        style="background-color: #{selectedColor}"
        title="#{selectedColor}"
      ></div>
    {/if}
  </div>
</div>
