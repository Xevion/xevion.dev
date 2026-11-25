<script lang="ts">
  import { resolve } from "$app/paths";
  import { css } from "styled-system/css";
  import { center } from "styled-system/patterns";

  let { data } = $props();

  const title = $derived(
    `${data.code} - ${data.message.charAt(0).toUpperCase() + data.message.slice(1)}`,
  );
</script>

<svelte:head>
  <title>{title}</title>
</svelte:head>

<main class="page-main">
  <div class={center({ minH: "100vh" })}>
    <div class={css({ mx: "4", maxW: "3xl", textAlign: "center" })}>
      <h1
        class={css({
          fontSize: "6xl",
          fontFamily: "hanken",
          fontWeight: "black",
          color: "zinc.200",
          sm: { fontSize: "9xl" },
        })}
      >
        {data.code}
      </h1>
      <p
        class={css({
          fontSize: "2xl",
          color: "zinc.400",
          mb: "8",
          textTransform: "capitalize",
          sm: { fontSize: "3xl" },
        })}
      >
        {data.message}
      </p>

      <!-- Only show "Return home" for non-transient errors -->
      {#if !data.transient}
        <a
          href={resolve("/")}
          class={css({
            display: "inline-block",
            py: "2",
            px: "4",
            bg: "zinc.900",
            color: "zinc.50",
            textDecoration: "none",
            rounded: "sm",
            transition: "colors",
            _hover: { bg: "zinc.800" },
          })}
        >
          Return home
        </a>
      {/if}
    </div>
  </div>
</main>
