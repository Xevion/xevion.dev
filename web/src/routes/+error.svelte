<script lang="ts">
  import AppWrapper from "$lib/components/AppWrapper.svelte";
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

<AppWrapper>
  <div class="flex min-h-screen items-center justify-center">
    <div class="mx-4 max-w-2xl text-center">
      <h1 class="mb-4 font-hanken text-8xl text-zinc-200">{status}</h1>
      <p class="mb-8 text-2xl text-zinc-400">{message}</p>
      {#if showHomeLink}
        <a
          href="/"
          class="inline-block rounded-sm bg-zinc-900 px-4 py-2 text-zinc-100 transition-colors hover:bg-zinc-800"
        >
          Return home
        </a>
      {/if}
    </div>
  </div>
</AppWrapper>
