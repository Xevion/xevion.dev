<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { page } from "$app/stores";
  import Button from "$lib/components/admin/Button.svelte";
  import Input from "$lib/components/admin/Input.svelte";
  import AppWrapper from "$lib/components/AppWrapper.svelte";
  import { authStore } from "$lib/stores/auth.svelte";

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
      console.error("Login error:", err);
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>Admin Login | xevion.dev</title>
</svelte:head>

<AppWrapper>
  <div class="flex min-h-screen items-center justify-center px-4">
    <div class="w-full max-w-md space-y-4">
      <!-- Login Form -->
      <div class="rounded-lg bg-admin-panel p-8 shadow-2xl shadow-zinc-500/20">
        <form onsubmit={handleSubmit} class="space-y-6">
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
              class="rounded-md bg-red-500/10 border border-red-500/20 p-3 text-sm text-red-400"
            >
              {error}
            </div>
          {/if}

          <Button
            type="submit"
            variant="primary"
            class="w-full"
            disabled={loading || !username || !password}
          >
            {loading ? "Signing in..." : "Sign in"}
          </Button>
        </form>
      </div>

      <!-- Back to site link -->
      <div class="text-center">
        <a
          href={resolve("/")}
          class="text-sm text-admin-text-muted hover:text-admin-text transition-colors"
        >
          ← Back to site
        </a>
      </div>
    </div>
  </div>
</AppWrapper>
