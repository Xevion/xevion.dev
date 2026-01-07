<script lang="ts">
  import { cn } from "$lib/utils";

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

  const inputStyles =
    "block w-full rounded-md border border-admin-border bg-admin-bg-secondary px-3 py-2 text-sm text-admin-text placeholder:text-admin-text-muted focus:border-admin-accent focus:outline-none focus:ring-1 focus:ring-admin-accent disabled:cursor-not-allowed disabled:opacity-50 transition-colors";

  const errorStyles = $derived(
    error ? "border-red-500 focus:border-red-500 focus:ring-red-500" : "",
  );

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

<div class={cn("space-y-1.5", className)}>
  {#if label}
    <label for={inputId} class="block text-sm font-medium text-admin-text">
      {label}
      {#if required}
        <span class="text-red-500">*</span>
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
      class={cn(inputStyles, errorStyles, "resize-y")}
      oninput={handleInput}
    ></textarea>
  {:else if type === "select"}
    <select
      id={inputId}
      bind:value
      {disabled}
      {required}
      class={cn(inputStyles, errorStyles)}
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
      class={cn(inputStyles, errorStyles)}
      oninput={handleInput}
    />
  {/if}

  {#if error}
    <p class="text-xs text-red-500">{error}</p>
  {/if}

  {#if help && !error}
    <p class="text-xs text-admin-text-muted">{help}</p>
  {/if}
</div>
