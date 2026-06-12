<script lang="ts" module>
  import seedrandom from "seedrandom";

  type MarkType = "dot" | "cross" | "plus" | "dash";
  interface Mark {
    x: number;
    y: number;
    type: MarkType;
    size: number;
    accent: boolean;
    opacity: number;
    angle?: number;
  }
  interface CoverField {
    w: number;
    h: number;
    marks: Mark[];
  }

  // Deterministic mark-field seeded by the project name/slug, echoing the site's
  // flow-field background. One of five motifs is chosen per seed, so every project
  // gets a stable, unique fingerprint. Ported verbatim from the design prototype.
  function cvBuild(
    seed: string,
    cols: number,
    rows: number,
    cell: number,
  ): CoverField {
    const rnd = seedrandom(seed);
    const w = cols * cell;
    const h = rows * cell;
    const variant = Math.floor(rnd() * 5);
    const marks: Mark[] = [];
    const acc = () => rnd() < 0.1;

    if (variant === 0) {
      // flow field — grid of dots/crosses pushed along a shared direction
      const a = rnd() * Math.PI * 2;
      const fx = Math.cos(a);
      const fy = Math.sin(a);
      for (let r = 0; r < rows; r++)
        for (let c = 0; c < cols; c++) {
          if (rnd() < 0.3) continue;
          const cx = c * cell + cell / 2;
          const cy = r * cell + cell / 2;
          const d = (rnd() - 0.5) * cell * 0.5;
          const cross = rnd() < 0.32;
          marks.push({
            x: cx + fx * d,
            y: cy + fy * d,
            type: cross ? "cross" : "dot",
            size: cross ? 3.4 : 2.4,
            accent: acc(),
            opacity: 0.32 + rnd() * 0.5,
          });
        }
    } else if (variant === 1) {
      // scatter — sparse plus/cross/dot of varying size
      const n = Math.floor(cols * rows * 0.5);
      for (let i = 0; i < n; i++) {
        const t = rnd();
        const type: MarkType = t < 0.2 ? "plus" : t < 0.36 ? "cross" : "dot";
        const size = type === "dot" ? 1.5 + rnd() * 2.3 : 3 + rnd() * 2.4;
        marks.push({
          x: rnd() * w,
          y: rnd() * h,
          type,
          size,
          accent: acc(),
          opacity: 0.28 + rnd() * 0.55,
        });
      }
    } else if (variant === 2) {
      // rings — concentric dot circles
      const cx = w / 2;
      const cy = h / 2;
      const maxR = Math.min(w, h) * 0.46;
      const ringCount = 3 + Math.floor(rnd() * 3);
      for (let k = 1; k <= ringCount; k++) {
        const radius = (maxR * k) / ringCount;
        const count = Math.max(
          6,
          Math.floor((2 * Math.PI * radius) / (cell * 0.7)),
        );
        const phase = rnd() * Math.PI * 2;
        for (let j = 0; j < count; j++) {
          const ang = phase + (j / count) * Math.PI * 2;
          const x = cx + Math.cos(ang) * radius;
          const y = cy + Math.sin(ang) * radius;
          if (x < 2 || x > w - 2 || y < 2 || y > h - 2) continue;
          const cross = rnd() < 0.16;
          marks.push({
            x,
            y,
            type: cross ? "cross" : "dot",
            size: cross ? 2.8 : 2.1,
            accent: acc(),
            opacity: 0.4 + rnd() * 0.45,
          });
        }
      }
    } else if (variant === 3) {
      // wave — rows of dots riding a sine
      const rowsN = Math.max(3, Math.round(rows * 0.9));
      const amp = cell * 0.7 + rnd() * cell * 0.6;
      const freq = (0.6 + rnd() * 0.8) / cell;
      const phase = rnd() * Math.PI * 2;
      const stepX = cell * 0.72;
      for (let ri = 0; ri < rowsN; ri++) {
        const baseY = (ri + 0.5) * (h / rowsN);
        for (let x = cell * 0.4; x < w; x += stepX) {
          const y = baseY + Math.sin(x * freq + phase + ri * 0.5) * amp * 0.4;
          const cross = rnd() < 0.14;
          marks.push({
            x,
            y,
            type: cross ? "cross" : "dot",
            size: cross ? 2.6 : 2.1,
            accent: acc(),
            opacity: 0.32 + rnd() * 0.45,
          });
        }
      }
    } else {
      // hatch — diagonal dashes on a grid
      const ang = (rnd() < 0.5 ? 1 : -1) * (Math.PI / 4);
      for (let r = 0; r < rows; r++)
        for (let c = 0; c < cols; c++) {
          if (rnd() < 0.3) continue;
          marks.push({
            x: c * cell + cell / 2,
            y: r * cell + cell / 2,
            type: "dash",
            size: cell * 0.52,
            angle: ang,
            accent: acc(),
            opacity: 0.28 + rnd() * 0.5,
          });
        }
    }
    return { w, h, marks };
  }
</script>

<script lang="ts">
  interface Props {
    seed: string;
    accent?: string;
    cols?: number;
    rows?: number;
    cell?: number;
    monogram?: string;
  }

  let {
    seed,
    accent = "#b7410e",
    cols = 14,
    rows = 8,
    cell = 22,
    monogram,
  }: Props = $props();

  const base = "#c8c8cc";
  const field = $derived(cvBuild(seed, cols, rows, cell));

  // accent marks sit slightly more opaque than the grayscale field
  const op = (m: Mark) =>
    m.accent ? Math.min(0.95, m.opacity + 0.22) : m.opacity;
  const color = (m: Mark) => (m.accent ? accent : base);
</script>

<svg
  viewBox="0 0 {field.w} {field.h}"
  preserveAspectRatio="xMidYMid slice"
  width="100%"
  height="100%"
  style="display: block"
  aria-hidden="true"
>
  {#if monogram}
    <text
      x={field.w / 2}
      y={field.h / 2}
      text-anchor="middle"
      dominant-baseline="central"
      font-family="'Schibsted Grotesk Variable', sans-serif"
      font-weight="800"
      font-size={field.h * 0.92}
      fill={accent}
      opacity="0.05"
      style="text-transform: lowercase">{monogram}</text
    >
  {/if}
  {#each field.marks as m, i (i)}
    {#if m.type === "dot"}
      <circle cx={m.x} cy={m.y} r={m.size} fill={color(m)} opacity={op(m)} />
    {:else if m.type === "cross"}
      <g
        stroke={color(m)}
        stroke-width="1.4"
        opacity={op(m)}
        stroke-linecap="round"
      >
        <line
          x1={m.x - m.size}
          y1={m.y - m.size}
          x2={m.x + m.size}
          y2={m.y + m.size}
        />
        <line
          x1={m.x - m.size}
          y1={m.y + m.size}
          x2={m.x + m.size}
          y2={m.y - m.size}
        />
      </g>
    {:else if m.type === "plus"}
      <g
        stroke={color(m)}
        stroke-width="1.5"
        opacity={op(m)}
        stroke-linecap="round"
      >
        <line x1={m.x - m.size} y1={m.y} x2={m.x + m.size} y2={m.y} />
        <line x1={m.x} y1={m.y - m.size} x2={m.x} y2={m.y + m.size} />
      </g>
    {:else}
      <line
        x1={m.x - Math.cos(m.angle ?? 0) * (m.size / 2)}
        y1={m.y - Math.sin(m.angle ?? 0) * (m.size / 2)}
        x2={m.x + Math.cos(m.angle ?? 0) * (m.size / 2)}
        y2={m.y + Math.sin(m.angle ?? 0) * (m.size / 2)}
        stroke={color(m)}
        stroke-width="1.6"
        opacity={op(m)}
        stroke-linecap="round"
      />
    {/if}
  {/each}
</svg>
