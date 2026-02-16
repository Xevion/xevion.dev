<script lang="ts">
  import { css, cva, cx } from "styled-system/css";
  import { flex, hstack } from "styled-system/patterns";
  import { page } from "$app/stores";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import { iconSm, iconMd } from "$lib/styles/admin";
  import IconLayoutDashboard from "~icons/lucide/layout-dashboard";
  import IconFolder from "~icons/lucide/folder";
  import IconTags from "~icons/lucide/tags";
  import IconList from "~icons/lucide/list";
  import IconSettings from "~icons/lucide/settings";
  import IconArrowLeft from "~icons/lucide/arrow-left";
  import IconLogOut from "~icons/lucide/log-out";
  import IconMenu from "~icons/lucide/menu";
  import IconX from "~icons/lucide/x";

  interface Props {
    projectCount?: number;
    tagCount?: number;
    onlogout?: () => void;
  }

  let { projectCount = 0, tagCount = 0, onlogout }: Props = $props();

  let mobileMenuOpen = $state(false);

  interface NavItem {
    href: string;
    label: string;
    icon: import("svelte").Component;
    badge?: number;
  }

  const navItems = $derived<NavItem[]>([
    { href: "/admin", label: "Dashboard", icon: IconLayoutDashboard },
    {
      href: "/admin/projects",
      label: "Projects",
      icon: IconFolder,
      badge: projectCount,
    },
    { href: "/admin/tags", label: "Tags", icon: IconTags, badge: tagCount },
    { href: "/admin/events", label: "Events", icon: IconList },
    { href: "/admin/settings", label: "Settings", icon: IconSettings },
  ]);

  const pathname = $derived($page.url.pathname as string);

  function isActive(href: string): boolean {
    if (href === "/admin") {
      return pathname === "/admin";
    }
    return pathname.startsWith(href);
  }

  function handleLogout() {
    onlogout?.();
  }

  const navLink = cva({
    base: {
      display: "flex",
      alignItems: "center",
      gap: "3",
      rounded: "md",
      px: "3",
      py: "2",
      fontSize: "sm",
      fontWeight: "medium",
      transition: "all",
      position: "relative",
    },
    variants: {
      state: {
        active: {
          bg: "admin.surfaceHover",
          color: "admin.text",
          _before: {
            content: '""',
            position: "absolute",
            left: "0",
            top: "1",
            bottom: "1",
            w: "0.5",
            bg: "admin.accent",
            borderTopRightRadius: "sm",
            borderBottomRightRadius: "sm",
          },
        },
        inactive: {
          color: "admin.textMuted",
          _hover: { color: "admin.text", bg: "admin.surfaceHover/50" },
        },
      },
    },
    defaultVariants: {
      state: "inactive",
    },
  });

  const bottomLink = hstack({
    gap: "3",
    rounded: "md",
    px: "3",
    py: "2",
    fontSize: "sm",
    fontWeight: "medium",
    color: "admin.textMuted",
    transition: "all",
    _hover: { color: "admin.text", bg: "admin.surfaceHover/50" },
  });
</script>

<!-- Mobile menu button -->
<button
  class={css({
    position: "fixed",
    top: "4",
    right: "4",
    zIndex: 50,
    lg: { display: "none" },
    rounded: "md",
    bg: "admin.surface",
    p: "2",
    color: "admin.text",
    borderWidth: "1px",
    borderColor: "admin.border",
  })}
  onclick={() => (mobileMenuOpen = !mobileMenuOpen)}
  aria-label="Toggle menu"
>
  {#if mobileMenuOpen}
    <IconX class={iconMd} />
  {:else}
    <IconMenu class={iconMd} />
  {/if}
</button>

<!-- Sidebar -->
<aside
  class={cx(
    css({
      position: "fixed",
      left: "0",
      top: "0",
      zIndex: 40,
      h: "100vh",
      w: "64",
      borderRightWidth: "1px",
      borderColor: "admin.border",
      bg: "admin.bg",
      transition: "transform",
      lg: { transform: "translateX(0)" },
    }),
    mobileMenuOpen
      ? css({ transform: "translateX(0)" })
      : css({ transform: "translateX(-100%)" }),
  )}
>
  <div class={flex({ direction: "column", h: "full" })}>
    <!-- Logo -->
    <div
      class={hstack({
        justify: "space-between",
        gap: "0",
        borderBottomWidth: "1px",
        borderColor: "admin.border",
        px: "4",
        py: "5",
      })}
    >
      <h1
        class={css({
          fontSize: "base",
          fontWeight: "semibold",
          color: "admin.text",
        })}
      >
        xevion.dev
        <span
          class={css({
            fontSize: "xs",
            fontWeight: "normal",
            color: "admin.textMuted",
            ml: "1.5",
          })}>Admin</span
        >
      </h1>
      <ThemeToggle />
    </div>

    <!-- Navigation -->
    <nav class={css({ flex: "1", spaceY: "0.5", p: "3" })}>
      {#each navItems as item (item.href)}
        <a
          href={item.href}
          class={navLink({
            state: isActive(item.href) ? "active" : "inactive",
          })}
        >
          <item.icon class={cx(iconSm, css({ flexShrink: "0" }))} />
          <span class={css({ flex: "1" })}>{item.label}</span>
          {#if item.badge}
            <span class={css({ fontSize: "xs", color: "admin.textMuted" })}>
              {item.badge}
            </span>
          {/if}
        </a>
      {/each}
    </nav>

    <!-- Bottom actions -->
    <div
      class={css({
        spaceY: "0.5",
        borderTopWidth: "1px",
        borderColor: "admin.border",
        bg: "admin.surface/50",
        p: "3",
      })}
    >
      <a href="/" class={bottomLink}>
        <IconArrowLeft class={iconSm} />
        <span>Back to Site</span>
      </a>
      <button onclick={handleLogout} class={cx(bottomLink, css({ w: "full" }))}>
        <IconLogOut class={iconSm} />
        <span>Logout</span>
      </button>
    </div>
  </div>
</aside>

<!-- Backdrop for mobile -->
{#if mobileMenuOpen}
  <div
    class={css({
      position: "fixed",
      inset: "0",
      zIndex: 30,
      bg: "black/50",
      lg: { display: "none" },
    })}
    onclick={() => (mobileMenuOpen = false)}
    onkeydown={(e) => e.key === "Escape" && (mobileMenuOpen = false)}
    role="presentation"
    tabindex="-1"
  ></div>
{/if}
