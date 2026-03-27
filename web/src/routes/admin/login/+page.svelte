<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { page } from "$app/stores";
  import Button from "$lib/components/admin/Button.svelte";
  import Input from "$lib/components/admin/Input.svelte";
  import { authStore } from "$lib/stores/auth.svelte";
  import { getLogger } from "@logtape/logtape";
  import { css, cx } from "styled-system/css";

  const logger = getLogger(["admin", "login"]);
  import { center } from "styled-system/patterns";

  let username = $state("");
  let password = $state("");
  let error = $state("");
  let loading = $state(false);

  async function handleSubmit(e: Event) {
    e.preventDefault();
    error = "";
    loading = true;

    try {
      const success = await authStore.login(username, password);

      if (success) {
        const nextUrl = $page.url.searchParams.get("next") || "/admin";
        goto(nextUrl);
      } else {
        error = "Invalid username or password";
      }
    } catch (err) {
      error = "An error occurred during login";
      logger.error("Login failed", { error: err });
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>Admin Login | xevion.dev</title>
</svelte:head>

<div
  class={css({
    pointerEvents: "none",
    position: "fixed",
    inset: "0",
    zIndex: -20,
    bg: "admin.bg",
  })}
></div>
<main class={cx("page-main", css({ color: "admin.text" }))}>
  <div class={center({ minH: "100vh", px: "4" })}>
    <div class={css({ w: "full", maxW: "md", spaceY: "4" })}>
      <!-- Login Form -->
      <div
        class={css({
          rounded: "lg",
          bg: "admin.surface",
          borderWidth: "1px",
          borderColor: "admin.border",
          p: "8",
          shadow: "2xl",
          shadowColor: "black/10",
          _dark: { shadowColor: "zinc.500/20" },
        })}
      >
        <form onsubmit={handleSubmit} class={css({ spaceY: "6" })}>
          <Input
            label="Username"
            type="text"
            bind:value={username}
            placeholder="admin"
            required
            disabled={loading}
          />

          <Input
            label="Password"
            type="password"
            bind:value={password}
            placeholder="••••••••"
            required
            disabled={loading}
          />

          {#if error}
            <div
              class={css({
                rounded: "md",
                bg: "red.500/10",
                borderWidth: "1px",
                borderColor: "red.500/20",
                p: "3",
                fontSize: "sm",
                color: "red.400",
              })}
            >
              {error}
            </div>
          {/if}

          <Button
            type="submit"
            variant="primary"
            class={css({ w: "full" })}
            disabled={loading || !username || !password}
          >
            {loading ? "Signing in..." : "Sign in"}
          </Button>
        </form>
      </div>

      <!-- Back to site link -->
      <div class={css({ textAlign: "center" })}>
        <a
          href={resolve("/")}
          class={css({
            fontSize: "sm",
            color: "admin.textMuted",
            _hover: { color: "admin.text" },
            transition: "colors",
          })}
        >
          ← Back to site
        </a>
      </div>
    </div>
  </div>
</main>
