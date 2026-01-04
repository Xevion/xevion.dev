<script lang="ts">
  import "../app.css";
  import "@fontsource-variable/inter";
  import "@fontsource/hanken-grotesk/900.css";
  import "@fontsource/schibsted-grotesk/400.css";
  import "@fontsource/schibsted-grotesk/500.css";
  import "@fontsource/schibsted-grotesk/600.css";

  let { children } = $props();
</script>

<svelte:head>
  <link rel="icon" href="/favicon.ico" />
  <title>Xevion.dev</title>
  <meta
    name="description"
    content="The personal website of Xevion, a full-stack software developer."
  />
</svelte:head>

{@render children()}
