<script lang="ts">
  import { resolve } from "$app/paths";
  import { page } from "$app/stores";

  const status = $derived($page.status);

  const messages: Record<number, string> = {
    404: "Page not found",
    405: "Method not allowed",
    500: "Something went wrong",
    502: "Service temporarily unavailable",
    503: "Service temporarily unavailable",
  };

  const message = $derived(messages[status] || "An error occurred");
  const showHomeLink = $derived(![502, 503].includes(status));
</script>

<svelte:head>
  <title>{status} - {message}</title>
</svelte:head>

<main class="page-main">
  <div class="flex min-h-screen items-center justify-center">
    <div class="mx-4 max-w-2xl text-center">
      <h1 class="mb-4 font-hanken text-8xl text-text-secondary">{status}</h1>
      <p class="mb-8 text-2xl text-text-tertiary">{message}</p>
      {#if showHomeLink}
        <a
          href={resolve("/")}
          class="inline-block rounded-sm bg-surface px-4 py-2 text-text-primary transition-colors hover:bg-surface-hover"
        >
          Return home
        </a>
      {/if}
    </div>
  </div>
</main>
