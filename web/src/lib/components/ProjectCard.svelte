<script lang="ts">
  import { cn } from "$lib/utils";
  import { telemetry } from "$lib/telemetry";
  import TagList from "./TagList.svelte";
  import type { AdminProject } from "$lib/admin-types";

  // Extended tag type with icon SVG for display
  type ProjectTag = { iconSvg?: string; name: string; color?: string };

  interface Props {
    project: AdminProject & {
      tags: ProjectTag[];
      clockIconSvg?: string;
    };
    class?: string;
  }

  let { project, class: className }: Props = $props();

  // Prefer demo URL, fallback to GitHub repo
  const projectUrl = $derived(
    project.demoUrl ||
      (project.githubRepo ? `https://github.com/${project.githubRepo}` : null),
  );

  const isLink = $derived(!!projectUrl);

  // Determine click action type for telemetry
  const clickAction = $derived(
    project.demoUrl ? "demo_click" : project.githubRepo ? "github_click" : null,
  );

  // Random seed generated once per component instance (changes on each page load)
  const randomSeed = Math.floor(Math.random() * 1000);

  // Randomly decide if this card shows video or image (~60% video)
  const isVideo = randomSeed % 10 < 6;

  // Sample video URLs
  const sampleVideos = [
    "https://storage.googleapis.com/gtv-videos-bucket/sample/ForBiggerBlazes.mp4",
    "https://storage.googleapis.com/gtv-videos-bucket/sample/ForBiggerEscapes.mp4",
    "https://storage.googleapis.com/gtv-videos-bucket/sample/ForBiggerFun.mp4",
    "https://storage.googleapis.com/gtv-videos-bucket/sample/ForBiggerJoyrides.mp4",
    "https://storage.googleapis.com/gtv-videos-bucket/sample/ForBiggerMeltdowns.mp4",
    "https://storage.googleapis.com/gtv-videos-bucket/sample/SubaruOutbackOnStreetAndDirt.mp4",
    "https://storage.googleapis.com/gtv-videos-bucket/sample/WeAreGoingOnBullrun.mp4",
    "https://res.cloudinary.com/demo/video/upload/w_640,h_360,c_fill/samples/elephants.mp4",
    "https://res.cloudinary.com/demo/video/upload/w_640,h_360,c_fill/samples/sea-turtle.mp4",
    "https://res.cloudinary.com/demo/video/upload/w_640,h_360,c_fill/dog.mp4",
    "https://res.cloudinary.com/demo/video/upload/w_640,h_360,c_fill/ski_jump.mp4",
    "https://res.cloudinary.com/demo/video/upload/w_640,h_360,c_fill/snow_horses.mp4",
  ];

  const videoUrl = sampleVideos[randomSeed % sampleVideos.length];

  // Randomized aspect ratios for images: [width, height]
  const aspectRatios: [number, number][] = [
    [400, 300], // 4:3 landscape
    [300, 400], // 3:4 portrait
    [400, 400], // 1:1 square
    [480, 270], // 16:9 landscape
    [270, 480], // 9:16 portrait
    [400, 240], // 5:3 wide landscape
    [240, 400], // 3:5 tall portrait
  ];
  const aspectIndex = randomSeed % aspectRatios.length;
  const [imgWidth, imgHeight] = aspectRatios[aspectIndex];

  const imageUrl = `https://picsum.photos/seed/${randomSeed}/${imgWidth}/${imgHeight}`;

  // Video element reference for play/pause control
  let videoElement: HTMLVideoElement | null = $state(null);

  function handleMouseEnter() {
    if (videoElement) {
      videoElement.play();
    }
  }

  function handleMouseLeave() {
    if (videoElement) {
      videoElement.pause();
    }
  }

  function handleClick() {
    if (clickAction && projectUrl) {
      telemetry.track({
        name: "project_interaction",
        properties: {
          action: clickAction,
          projectSlug: project.slug,
          projectName: project.name,
          targetUrl: projectUrl,
        },
      });
    }
  }

  // Shared classes for background media (image/video)
  const mediaBaseClasses =
    "media-mask-fade-left absolute right-0 top-0 h-full w-3/4 object-cover object-center";

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffMins < 1) return "just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffHours <= 48) return "yesterday";
    if (diffDays < 30) return `${diffDays}d ago`;
    if (diffDays < 365) return `${Math.floor(diffDays / 30)}mo ago`;
    return `${Math.floor(diffDays / 365)}y ago`;
  }
</script>

<svelte:element
  this={isLink ? "a" : "div"}
  href={isLink ? projectUrl : undefined}
  target={isLink ? "_blank" : undefined}
  rel={isLink ? "noopener noreferrer" : undefined}
  onclick={handleClick}
  onmouseenter={handleMouseEnter}
  onmouseleave={handleMouseLeave}
  role={isLink ? undefined : "article"}
  class={cn(
    "group relative flex h-44 flex-col gap-2.5 rounded-lg border border-zinc-200 dark:border-zinc-800 bg-zinc-50 dark:bg-zinc-900/50 p-3 overflow-hidden",
    {
      "transition-all hover:border-zinc-300 dark:hover:border-zinc-700 hover:bg-zinc-100/80 dark:hover:bg-zinc-800/50":
        isLink,
      className: true,
    },
  )}
>
  <!-- Background media layer -->
  <div
    class="pointer-events-none absolute inset-0 opacity-25 group-hover:opacity-40"
    style="transition: opacity 300ms ease-in-out;"
    aria-hidden="true"
  >
    {#if isVideo}
      <video
        bind:this={videoElement}
        src={videoUrl}
        class={cn(mediaBaseClasses, "grayscale group-hover:grayscale-0")}
        style="transition: filter 300ms ease-in-out;"
        muted
        loop
        playsinline
        preload="metadata"
      ></video>
    {:else}
      <img src={imageUrl} alt="" class={mediaBaseClasses} loading="lazy" />
    {/if}
  </div>

  <!-- Content layer -->
  <div
    class="relative z-10 flex flex-col gap-1 group-hover:opacity-80"
    style="transition: opacity 300ms ease-in-out;"
  >
    <div class="flex items-start justify-between gap-2">
      <h3
        class={cn(
          "truncate font-medium text-lg sm:text-base text-zinc-900 dark:text-zinc-100",
          isLink &&
            "transition-colors group-hover:text-zinc-950 dark:group-hover:text-white",
        )}
      >
        {project.name}
      </h3>
      <span class="shrink-0 sm:text-[0.83rem] text-zinc-600 dark:text-zinc-300">
        {formatDate(project.lastActivity)}
      </span>
    </div>
    <p
      class="line-clamp-3 sm:text-sm leading-relaxed text-zinc-600 dark:text-zinc-400"
    >
      {project.shortDescription}
    </p>
  </div>

  <!-- Tags layer -->
  <TagList
    tags={project.tags}
    maxRows={2}
    class="relative z-10 mt-auto group-hover:opacity-90"
    style="transition: opacity 300ms ease-in-out;"
  />
</svelte:element>
