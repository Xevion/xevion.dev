<script lang="ts">
  import { onMount } from "svelte";
  import { browser } from "$app/environment";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  let imageKey = $state(0);
  let loading = $state(false);

  // Auto-reload image when HMR updates
  onMount(() => {
    if (!browser) return undefined;

    // Trigger reload when HMR updates
    if (import.meta.hot) {
      import.meta.hot.on("vite:afterUpdate", () => {
        imageKey++;
      });
    }
  });

  async function regenerate() {
    loading = true;
    imageKey++;
    loading = false;
  }

  const imageUrl = $derived(
    `/internal/ogp/generate?${new URLSearchParams(data.spec).toString()}&_=${imageKey}`,
  );
</script>

<svelte:head>
  <title>OG Image Preview - {data.title}</title>
</svelte:head>

<div class="container">
  <div class="header">
    <h1>OG Image Preview</h1>
    <div class="controls">
      <button onclick={regenerate} disabled={loading}>
        {loading ? "Loading..." : "Refresh"}
      </button>
    </div>
  </div>

  <div class="info">
    <p><strong>Type:</strong> {data.spec.type}</p>
    {#if data.spec.type === "project"}
      <p><strong>ID:</strong> {data.spec.id}</p>
    {/if}
    <p class="hint">
      Image auto-reloads when server updates (HMR) or every 2 seconds
    </p>
  </div>

  <div class="preview">
    <img src={imageUrl} alt="Preview" width="1200" height="630" class:loading />
  </div>

  <div class="examples">
    <h2>Example URLs:</h2>
    <ul>
      <li><a href="/internal/ogp?type=index">Index page</a></li>
      <li><a href="/internal/ogp?type=projects">Projects page</a></li>
      <li>
        <a href="/internal/ogp?type=project&id=example-id"
          >Project page (needs valid ID)</a
        >
      </li>
    </ul>
  </div>
</div>

<style>
  .container {
    max-width: 1400px;
    margin: 0 auto;
    padding: 2rem;
    font-family:
      system-ui,
      -apple-system,
      sans-serif;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
  }

  h1 {
    margin: 0;
    font-size: 2rem;
    font-weight: 700;
  }

  .controls button {
    padding: 0.5rem 1rem;
    font-size: 1rem;
    background: #000;
    color: #fff;
    border: none;
    border-radius: 0.25rem;
    cursor: pointer;
    transition: opacity 0.2s;
  }

  .controls button:hover:not(:disabled) {
    opacity: 0.8;
  }

  .controls button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .info {
    background: #f5f5f5;
    padding: 1rem;
    border-radius: 0.5rem;
    margin-bottom: 2rem;
  }

  .info p {
    margin: 0.5rem 0;
  }

  .hint {
    color: #666;
    font-size: 0.875rem;
    font-style: italic;
  }

  .preview {
    border: 2px solid #e5e5e5;
    border-radius: 0.5rem;
    overflow: hidden;
    background: #fafafa;
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 2rem;
  }

  .preview img {
    max-width: 100%;
    height: auto;
    display: block;
    box-shadow:
      0 4px 6px -1px rgb(0 0 0 / 0.1),
      0 2px 4px -2px rgb(0 0 0 / 0.1);
    transition: opacity 0.2s;
  }

  .preview img.loading {
    opacity: 0.6;
  }

  .examples {
    margin-top: 3rem;
    padding-top: 2rem;
    border-top: 1px solid #e5e5e5;
  }

  .examples h2 {
    font-size: 1.25rem;
    margin-bottom: 1rem;
  }

  .examples ul {
    list-style: none;
    padding: 0;
  }

  .examples li {
    margin: 0.5rem 0;
  }

  .examples a {
    color: #0066cc;
    text-decoration: none;
  }

  .examples a:hover {
    text-decoration: underline;
  }
</style>
