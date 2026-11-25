<script lang="ts">
  interface Props {
    src: string;
    onload?: () => void;
    class?: string;
  }

  let { src, onload, class: className }: Props = $props();

  let canvasRef: HTMLCanvasElement | null = $state(null);
  let loaded = $state(false);

  $effect(() => {
    if (!canvasRef || !src || loaded) return;

    const video = document.createElement("video");
    video.crossOrigin = "anonymous";
    video.muted = true;
    video.playsInline = true;
    video.preload = "metadata";

    video.onloadeddata = () => {
      // Seek to 0.1s to avoid black frames
      video.currentTime = 0.1;
    };

    video.onseeked = () => {
      if (!canvasRef) return;

      const ctx = canvasRef.getContext("2d");
      if (ctx) {
        // Set canvas size to match video
        canvasRef.width = video.videoWidth;
        canvasRef.height = video.videoHeight;

        // Draw the frame
        ctx.drawImage(video, 0, 0, video.videoWidth, video.videoHeight);
        loaded = true;
        onload?.();
      }

      // Clean up
      video.src = "";
      video.load();
    };

    video.onerror = () => {
      // Still call onload to remove loading state
      loaded = true;
      onload?.();
    };

    video.src = src;
  });
</script>

<canvas
  bind:this={canvasRef}
  class={className ?? "absolute inset-0 w-full h-full object-cover"}
></canvas>
