import { cn } from "@/utils/helpers";
import { p5i, P5I } from "p5i";
import React, { useEffect, useRef } from "react";

interface DotsProps {
  className?: string;
}

const Dots = ({ className }: DotsProps) => {
  const canvasRef = useRef<HTMLDivElement | null>(null);
  const w = useRef(window.innerWidth);
  const h = useRef(window.innerHeight);

  const {
    mount,
    unmount,
    background,
    stroke,
    noise,
    resizeCanvas,
    cos,
    sin,
    TWO_PI,
  } = p5i();

  const offsetY = window.scrollY;

  const SCALE = 400;
  const LENGTH = 3;
  const SPACING = 20;
  const TARGET_FRAMERATE = 5;
  const TIMESCALE = 2;
  const OPACITY = 0.7;

  function getForceOnPoint(x: number, y: number, z: number) {
    return (noise(x / SCALE, y / SCALE, z) - 0.5) * 2 * TWO_PI;
  }

  const pointIds = new Set<string>();
  const points: { x: number; y: number; opacity: number }[] = [];

  function addPoints() {
    for (let x = -SPACING / 2; x < w.current + SPACING; x += SPACING) {
      for (
        let y = -SPACING / 2;
        y < h.current + offsetY + SPACING;
        y += SPACING
      ) {
        const id = `${x}-${y}`;
        if (pointIds.has(id)) continue;
        pointIds.add(id);

        points.push({ x, y, opacity: Math.random() * 0.5 + 0.5 });
      }
    }
  }

  function setup({
    createCanvas,
    stroke,
    frameRate,
    background,
    noFill,
    noiseSeed,
  }: P5I) {
    createCanvas(w.current, h.current);
    background("#000000");
    stroke("rgba(170, 170, 170, 0.05)");
    noFill();

    frameRate(TARGET_FRAMERATE);
    noiseSeed(Date.now());

    addPoints();
  }

  function draw({ circle, frameCount }: P5I) {
    background("#000000");
    const t = (frameCount / 80) * TIMESCALE;

    // if (frameCount % 10000) console.log(frameRate());

    for (const p of points) {
      const rad = getForceOnPoint(p.x, p.y, t);
      const length = (noise(p.x / SCALE, p.y / SCALE, t * 2) + 0.5) * LENGTH;
      const nx = p.x + cos(rad) * length;
      const ny = p.y + sin(rad) * length;

      // const center_distance = Math.sqrt((x - w / 2) ** 2 + (y - h / 2) ** 2);
      // if (center_distance < 350)
      //     opacity = 0;
      //     opacity =
      stroke(
        200,
        200,
        200,
        (Math.abs(cos(rad)) * 0.8 + 0.1) * p.opacity * 255 * OPACITY,
      );
      circle(nx, ny - offsetY, 1);
    }
  }

  function restart() {
    if (canvasRef.current) {
      mount(canvasRef.current, { setup, draw });
    }
  }

  useEffect(() => {
    restart();

    const handleResize = () => {
      w.current = window.innerWidth;
      h.current = window.innerHeight;
      resizeCanvas(w.current, h.current);
      addPoints();
    };

    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
      unmount();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div
      ref={canvasRef}
      className={cn(
        "pointer-events-none fixed bottom-0 left-0 right-0 top-0 z-0 opacity-0",
        className,
      )}
    />
  );
};

export default Dots;
