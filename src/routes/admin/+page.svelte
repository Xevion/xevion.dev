<script lang="ts">
  import Button from "#lib/components/admin/Button.svelte";
  import IconPlus from "~icons/lucide/plus";
  import { css, cx } from "styled-system/css";
  import { wrap } from "styled-system/patterns";
  import {
    pageTitleClass,
    pageDescriptionClass,
    iconSm,
  } from "#lib/styles/admin.ts";
</script>

<svelte:head>
  <title>Dashboard | Admin</title>
</svelte:head>

<div class={css({ spaceY: "6" })}>
  <div>
    <h1 class={pageTitleClass}>Dashboard</h1>
    <p class={cx(pageDescriptionClass, css({ mt: "1" }))}>
      Overview of your portfolio
    </p>
  </div>

  <div class={wrap({ gap: "3" })}>
    <Button variant="primary" href="/admin/projects/new">
      <IconPlus class={cx(iconSm, css({ mr: "2" }))} />
      New Project
    </Button>
    <Button variant="secondary" href="/admin/projects">
      View All Projects
    </Button>
    <Button variant="secondary" href="/admin/tags">Manage Tags</Button>
  </div>
</div>
