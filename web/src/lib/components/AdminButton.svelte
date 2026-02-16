<script lang="ts">
  import { css, cx } from "styled-system/css";
  import { authStore } from "$lib/stores/auth.svelte";
  import { iconButton } from "styled-system/recipes";
  import { iconMd } from "$lib/styles/admin";
  import IconLayoutDashboard from "~icons/lucide/layout-dashboard";
</script>

{#if authStore.isAuthenticated}
  <a href="/admin" aria-label="Admin dashboard" class={iconButton()}>
    <IconLayoutDashboard
      class={cx(
        iconMd,
        css({ color: "zinc.600", _dark: { color: "zinc.400" } }),
      )}
    />
  </a>
{/if}
