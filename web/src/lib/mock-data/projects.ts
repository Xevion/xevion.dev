export interface MockProjectTag {
  name: string;
  icon: string; // Icon identifier like "simple-icons:rust"
  color?: string; // Hex color without hash
  iconSvg?: string; // Pre-rendered SVG (populated server-side)
}

export interface MockProject {
  id: string;
  name: string;
  description: string;
  url: string;
  tags: MockProjectTag[];
  lastActivity: string;
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
      { name: "Rust", icon: "simple-icons:rust", color: "f97316" },
      { name: "SvelteKit", icon: "simple-icons:svelte", color: "f43f5e" },
      { name: "PostgreSQL", icon: "cib:postgresql", color: "3b82f6" },
    ],
    lastActivity: "2026-01-06T22:12:37Z",
  },
  {
    id: "2",
    name: "historee",
    description:
      "Powerful browser history analyzer for visualizing and understanding web browsing patterns across multiple browsers.",
    url: "https://github.com/Xevion/historee",
    tags: [
      { name: "Rust", icon: "simple-icons:rust", color: "f97316" },
      { name: "CLI", icon: "lucide:terminal", color: "a1a1aa" },
      { name: "Analytics", icon: "lucide:bar-chart-3", color: "10b981" },
    ],
    lastActivity: "2026-01-06T06:01:27Z",
  },
  {
    id: "3",
    name: "satori-html",
    description:
      "HTML adapter for Vercel's Satori library, enabling generation of beautiful social card images from HTML markup.",
    url: "https://github.com/Xevion/satori-html",
    tags: [
      { name: "TypeScript", icon: "simple-icons:typescript", color: "3b82f6" },
      { name: "NPM", icon: "simple-icons:npm", color: "ec4899" },
      { name: "Graphics", icon: "lucide:image", color: "a855f7" },
    ],
    lastActivity: "2026-01-05T20:23:07Z",
  },
  {
    id: "4",
    name: "byte-me",
    description:
      "Cross-platform media bitrate visualizer with real-time analysis. Built with Tauri for native performance and modern UI.",
    url: "https://github.com/Xevion/byte-me",
    tags: [
      { name: "Rust", icon: "simple-icons:rust", color: "f97316" },
      { name: "Tauri", icon: "simple-icons:tauri", color: "14b8a6" },
      { name: "Desktop", icon: "lucide:monitor", color: "6366f1" },
      { name: "Media", icon: "lucide:video", color: "f43f5e" },
    ],
    lastActivity: "2026-01-05T05:09:09Z",
  },
  {
    id: "5",
    name: "rdap",
    description:
      "Modern RDAP query client for domain registration data lookup. Clean interface built with static Next.js for instant loads.",
    url: "https://github.com/Xevion/rdap",
    tags: [
      { name: "TypeScript", icon: "simple-icons:typescript", color: "3b82f6" },
      { name: "Next.js", icon: "simple-icons:nextdotjs", color: "a1a1aa" },
      { name: "Networking", icon: "lucide:network", color: "0ea5e9" },
    ],
    lastActivity: "2026-01-05T10:36:55Z",
  },
  {
    id: "6",
    name: "rebinded",
    description:
      "Cross-platform key remapping daemon with per-application context awareness and intelligent stateful debouncing.",
    url: "https://github.com/Xevion/rebinded",
    tags: [
      { name: "Rust", icon: "simple-icons:rust", color: "f97316" },
      { name: "System", icon: "lucide:settings-2", color: "a1a1aa" },
      { name: "Cross-platform", icon: "lucide:globe", color: "22c55e" },
    ],
    lastActivity: "2026-01-01T00:34:09Z",
  },
];
