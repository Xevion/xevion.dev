<script lang="ts">
  import { resolve } from "$app/paths";
  import AppWrapper from "$components/AppWrapper.svelte";

  let { data } = $props();

  const title = $derived(
    `${data.code} - ${data.message.charAt(0).toUpperCase() + data.message.slice(1)}`,
  );
</script>

<svelte:head>
  <title>{title}</title>
</svelte:head>

<AppWrapper>
  <div class="min-h-screen flex items-center justify-center">
    <div class="mx-4 max-w-3xl text-center">
      <h1 class="text-6xl sm:text-9xl font-hanken font-black text-zinc-200">
        {data.code}
      </h1>
      <p class="text-2xl sm:text-3xl text-zinc-400 mb-8 capitalize">
        {data.message}
      </p>

      <!-- Only show "Return home" for non-transient errors -->
      {#if !data.transient}
        <a
          href={resolve("/")}
          class="inline-block py-2 px-4 bg-zinc-900 text-zinc-50 no-underline rounded-sm transition-colors hover:bg-zinc-800"
        >
          Return home
        </a>
      {/if}
    </div>
  </div>
</AppWrapper>
