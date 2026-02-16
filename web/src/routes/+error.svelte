<script lang="ts">
  import { resolve } from "$app/paths";
  import { page } from "$app/stores";
  import { css, cx } from "styled-system/css";
  import { center } from "styled-system/patterns";

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

<main class={cx("page-main")}>
  <div class={center({ minH: "100vh" })}>
    <div class={css({ mx: "4", maxW: "42rem", textAlign: "center" })}>
      <h1
        class={css({
          mb: "4",
          fontFamily: "hanken",
          fontSize: "8xl",
          color: "text.secondary",
        })}
      >
        {status}
      </h1>
      <p class={css({ mb: "8", fontSize: "2xl", color: "text.tertiary" })}>
        {message}
      </p>
      {#if showHomeLink}
        <a
          href={resolve("/")}
          class={css({
            display: "inline-block",
            rounded: "sm",
            bg: "surface",
            px: "4",
            py: "2",
            color: "text.primary",
            transition: "colors",
            _hover: { bg: "surface.secondary" },
          })}
        >
          Return home
        </a>
      {/if}
    </div>
  </div>
</main>
