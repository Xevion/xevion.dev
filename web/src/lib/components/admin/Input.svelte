<script lang="ts">
  import { css, cx } from "styled-system/css";
  import {
    labelClass,
    helpTextClass,
    errorTextClass,
    fieldWrapperClass,
    adminInputBase,
    adminInputError,
  } from "$lib/styles/admin";

  interface Props {
    label?: string;
    type?:
      | "text"
      | "number"
      | "email"
      | "password"
      | "url"
      | "textarea"
      | "select";
    value: string | number;
    placeholder?: string;
    disabled?: boolean;
    required?: boolean;
    error?: string;
    help?: string;
    class?: string;
    rows?: number;
    options?: Array<{ value: string; label: string }>;
    oninput?: (value: string | number) => void;
  }

  let {
    type = "text",
    value = $bindable(),
    label,
    placeholder,
    disabled = false,
    required = false,
    error,
    help,
    class: className,
    rows = 4,
    options = [],
    oninput,
  }: Props = $props();

  // Generate unique ID for accessibility
  const inputId = `input-${Math.random().toString(36).substring(2, 11)}`;

  const errorBorder = $derived(error ? adminInputError : "");

  function handleInput(e: Event) {
    const target = e.target as
      | HTMLInputElement
      | HTMLTextAreaElement
      | HTMLSelectElement;
    const newValue = type === "number" ? Number(target.value) : target.value;
    value = newValue;
    oninput?.(newValue);
  }
</script>

<div class={cx(fieldWrapperClass, className)}>
  {#if label}
    <label for={inputId} class={labelClass}>
      {label}
      {#if required}
        <span class={css({ color: "red.500" })}>*</span>
      {/if}
    </label>
  {/if}

  {#if type === "textarea"}
    <textarea
      id={inputId}
      bind:value
      {placeholder}
      {disabled}
      {required}
      {rows}
      class={cx(adminInputBase, errorBorder, css({ resize: "vertical" }))}
      oninput={handleInput}
    ></textarea>
  {:else if type === "select"}
    <select
      id={inputId}
      bind:value
      {disabled}
      {required}
      class={cx(adminInputBase, errorBorder)}
      onchange={handleInput}
    >
      {#if placeholder}
        <option value="" disabled>{placeholder}</option>
      {/if}
      {#each options as option (option.value)}
        <option value={option.value}>{option.label}</option>
      {/each}
    </select>
  {:else}
    <input
      id={inputId}
      {type}
      bind:value
      {placeholder}
      {disabled}
      {required}
      class={cx(adminInputBase, errorBorder)}
      oninput={handleInput}
    />
  {/if}

  {#if error}
    <p class={errorTextClass}>{error}</p>
  {/if}

  {#if help && !error}
    <p class={helpTextClass}>{help}</p>
  {/if}
</div>
