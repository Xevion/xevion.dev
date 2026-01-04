<script lang="ts">
  import AppWrapper from "$lib/components/AppWrapper.svelte";
  import Balancer from "svelte-wrap-balancer";
  import { cn } from "$lib/utils";

  let { data } = $props();
</script>

<AppWrapper>
  <div
    class="relative z-10 mx-auto grid grid-cols-1 justify-center gap-y-4 px-4 py-20 align-middle sm:grid-cols-2 md:max-w-[50rem] lg:max-w-[75rem] lg:grid-cols-3 lg:gap-y-9"
  >
    <div class="mb-3 text-center sm:col-span-2 md:mb-5 lg:col-span-3 lg:mb-7">
      <h1 class="pb-3 font-hanken text-4xl text-zinc-200 opacity-100 md:text-5xl">
        Projects
      </h1>
      <Balancer>
        <p class="text-lg text-zinc-400">
          created, maintained, or contributed to by me...
        </p>
      </Balancer>
    </div>

    {#each data.projects as project (project.id)}
      {@const links = project.links}
      {@const useAnchor = links.length > 0}
      {@const href = useAnchor ? links[0].url : undefined}

      <div class="max-w-fit">
        <svelte:element
          this={useAnchor ? "a" : "div"}
          {href}
          target={useAnchor ? "_blank" : undefined}
          rel={useAnchor ? "noreferrer" : undefined}
          title={project.name}
          class="flex items-center justify-start overflow-hidden rounded bg-black/10 pb-2.5 pl-3 pr-5 pt-1 text-zinc-400 transition-colors hover:bg-zinc-500/10 hover:text-zinc-50"
        >
          <div class="flex h-full w-14 items-center justify-center pr-5">
            <i
              class={cn(
                project.icon ?? "fa-heart",
                "fa-solid text-3xl text-opacity-80 saturate-0"
              )}
            ></i>
          </div>
          <div class="overflow-hidden">
            <span class="text-sm md:text-base lg:text-lg">
              {project.name}
            </span>
            <p
              class="truncate text-xs opacity-70 md:text-sm lg:text-base"
              title={project.shortDescription}
            >
              {project.shortDescription}
            </p>
          </div>
        </svelte:element>
      </div>
    {/each}
  </div>
</AppWrapper>
