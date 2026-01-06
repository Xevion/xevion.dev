export interface MockProjectTag {
  name: string;
  icon: string; // Icon identifier like "simple-icons:rust"
  iconSvg?: string; // Pre-rendered SVG (populated server-side)
}

export interface MockProject {
  id: string;
  name: string;
  description: string;
  url: string;
  tags: MockProjectTag[];
  updatedAt: string;
  clockIconSvg?: string; // Pre-rendered clock icon for "Updated" text
}

export const MOCK_PROJECTS: MockProject[] = [
  {
    id: "1",
    name: "xevion.dev",
    description:
      "Personal portfolio showcasing projects and technical expertise. Built with Rust backend, SvelteKit frontend, and PostgreSQL.",
    url: "https://github.com/Xevion/xevion.dev",
    tags: [
      { name: "Rust", icon: "simple-icons:rust" },
      { name: "SvelteKit", icon: "simple-icons:svelte" },
      { name: "PostgreSQL", icon: "cib:postgresql" },
    ],
    updatedAt: "2026-01-06T22:12:37Z",
  },
  {
    id: "2",
    name: "historee",
    description:
      "Powerful browser history analyzer for visualizing and understanding web browsing patterns across multiple browsers.",
    url: "https://github.com/Xevion/historee",
    tags: [
      { name: "Rust", icon: "simple-icons:rust" },
      { name: "CLI", icon: "lucide:terminal" },
      { name: "Analytics", icon: "lucide:bar-chart-3" },
    ],
    updatedAt: "2026-01-06T06:01:27Z",
  },
  {
    id: "3",
    name: "satori-html",
    description:
      "HTML adapter for Vercel's Satori library, enabling generation of beautiful social card images from HTML markup.",
    url: "https://github.com/Xevion/satori-html",
    tags: [
      { name: "TypeScript", icon: "simple-icons:typescript" },
      { name: "NPM", icon: "simple-icons:npm" },
      { name: "Graphics", icon: "lucide:image" },
    ],
    updatedAt: "2026-01-05T20:23:07Z",
  },
  {
    id: "4",
    name: "byte-me",
    description:
      "Cross-platform media bitrate visualizer with real-time analysis. Built with Tauri for native performance and modern UI.",
    url: "https://github.com/Xevion/byte-me",
    tags: [
      { name: "Rust", icon: "simple-icons:rust" },
      { name: "Tauri", icon: "simple-icons:tauri" },
      { name: "Desktop", icon: "lucide:monitor" },
      { name: "Media", icon: "lucide:video" },
    ],
    updatedAt: "2026-01-05T05:09:09Z",
  },
  {
    id: "5",
    name: "rdap",
    description:
      "Modern RDAP query client for domain registration data lookup. Clean interface built with static Next.js for instant loads.",
    url: "https://github.com/Xevion/rdap",
    tags: [
      { name: "TypeScript", icon: "simple-icons:typescript" },
      { name: "Next.js", icon: "simple-icons:nextdotjs" },
      { name: "Networking", icon: "lucide:network" },
    ],
    updatedAt: "2026-01-05T10:36:55Z",
  },
  {
    id: "6",
    name: "rebinded",
    description:
      "Cross-platform key remapping daemon with per-application context awareness and intelligent stateful debouncing.",
    url: "https://github.com/Xevion/rebinded",
    tags: [
      { name: "Rust", icon: "simple-icons:rust" },
      { name: "System", icon: "lucide:settings-2" },
      { name: "Cross-platform", icon: "lucide:globe" },
    ],
    updatedAt: "2026-01-01T00:34:09Z",
  },
];
