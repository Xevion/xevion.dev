<script lang="ts">
  import { enhance } from "$app/forms";
  import { resolve } from "$app/paths";
  import Button from "#lib/components/admin/Button.svelte";
  import {
    adminInputBase,
    adminInputError,
    errorTextClass,
    fieldWrapperClass,
    helpTextClass,
    labelClass,
  } from "#lib/styles/admin.ts";
  import { css, cx } from "styled-system/css";
  import { center } from "styled-system/patterns";
  import type { PageProps } from "./$types";

  let { form }: PageProps = $props();

  let working = $state(false);
  const errors = $derived<Record<string, string>>(form?.fieldErrors ?? {});
</script>

<svelte:head>
  <title>Set up | xevion.dev</title>
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
        <form
          method="POST"
          use:enhance={() => {
            working = true;
            return async ({ update }) => {
              await update({ reset: false });
              working = false;
            };
          }}
          class={css({ spaceY: "6" })}
        >
          <div>
            <h1
              class={css({ textStyle: "admin.pageTitle", color: "admin.text" })}
            >
              Create the admin account
            </h1>
            <p class={helpTextClass}>
              One-time setup. This page disappears once an account exists.
            </p>
          </div>

          <div class={fieldWrapperClass}>
            <label for="username" class={labelClass}>Username</label>
            <input
              id="username"
              name="username"
              type="text"
              autocomplete="username"
              required
              value={form?.username ?? ""}
              class={cx(adminInputBase, errors.username && adminInputError)}
            />
            {#if errors.username}<p class={errorTextClass}>
                {errors.username}
              </p>{/if}
          </div>

          <div class={fieldWrapperClass}>
            <label for="password" class={labelClass}>Password</label>
            <input
              id="password"
              name="password"
              type="password"
              autocomplete="new-password"
              minlength="12"
              required
              class={cx(adminInputBase, errors.password && adminInputError)}
            />
            {#if errors.password}
              <p class={errorTextClass}>{errors.password}</p>
            {:else}
              <p class={helpTextClass}>At least 12 characters.</p>
            {/if}
          </div>

          <div class={fieldWrapperClass}>
            <label for="confirmPassword" class={labelClass}>
              Confirm password
            </label>
            <input
              id="confirmPassword"
              name="confirmPassword"
              type="password"
              autocomplete="new-password"
              required
              class={cx(
                adminInputBase,
                errors.confirmPassword && adminInputError,
              )}
            />
            {#if errors.confirmPassword}
              <p class={errorTextClass}>{errors.confirmPassword}</p>
            {/if}
          </div>

          {#if form && "message" in form && form.message}
            <p class={errorTextClass}>{form.message}</p>
          {/if}

          <Button
            type="submit"
            variant="primary"
            class={css({ w: "full" })}
            disabled={working}
          >
            {working ? "Creating..." : "Create account"}
          </Button>
        </form>
      </div>

      <div class={css({ textAlign: "center" })}>
        <a
          href={resolve("/")}
          class={css({
            fontSize: "sm",
            color: "admin.textMuted",
            _hover: { color: "admin.text" },
          })}
        >
          Back to site
        </a>
      </div>
    </div>
  </div>
</main>
