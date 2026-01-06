import type {
  AdminProject,
  AdminTag,
  AdminTagWithCount,
  AdminEvent,
  AdminStats,
  CreateProjectData,
  UpdateProjectData,
  CreateTagData,
  UpdateTagData,
  SiteSettings,
  SiteIdentity,
  SocialLink,
  AdminPreferences,
} from "./admin-types";

// ============================================================================
// ADMIN API FUNCTIONS (Mocked for now, will be replaced with real API calls)
// ============================================================================

// Mock data storage (in-memory for now)
let MOCK_TAGS: AdminTag[] = [
  { id: "tag-1", slug: "rust", name: "Rust", createdAt: "2024-01-15T10:00:00Z" },
  { id: "tag-2", slug: "typescript", name: "TypeScript", createdAt: "2024-01-16T10:00:00Z" },
  { id: "tag-3", slug: "web", name: "Web", createdAt: "2024-01-17T10:00:00Z" },
  { id: "tag-4", slug: "cli", name: "CLI", createdAt: "2024-01-18T10:00:00Z" },
  { id: "tag-5", slug: "api", name: "API", createdAt: "2024-01-19T10:00:00Z" },
  { id: "tag-6", slug: "database", name: "Database", createdAt: "2024-01-20T10:00:00Z" },
  { id: "tag-7", slug: "svelte", name: "Svelte", createdAt: "2024-01-21T10:00:00Z" },
  { id: "tag-8", slug: "python", name: "Python", createdAt: "2024-01-22T10:00:00Z" },
  { id: "tag-9", slug: "machine-learning", name: "Machine Learning", createdAt: "2024-01-23T10:00:00Z" },
  { id: "tag-10", slug: "docker", name: "Docker", createdAt: "2024-01-24T10:00:00Z" },
  { id: "tag-11", slug: "kubernetes", name: "Kubernetes", createdAt: "2024-01-25T10:00:00Z" },
  { id: "tag-12", slug: "react", name: "React", createdAt: "2024-01-26T10:00:00Z" },
  { id: "tag-13", slug: "nextjs", name: "Next.js", createdAt: "2024-01-27T10:00:00Z" },
  { id: "tag-14", slug: "tailwind", name: "Tailwind CSS", createdAt: "2024-01-28T10:00:00Z" },
  { id: "tag-15", slug: "graphql", name: "GraphQL", createdAt: "2024-01-29T10:00:00Z" },
  { id: "tag-16", slug: "postgres", name: "PostgreSQL", createdAt: "2024-01-30T10:00:00Z" },
  { id: "tag-17", slug: "redis", name: "Redis", createdAt: "2024-01-31T10:00:00Z" },
  { id: "tag-18", slug: "aws", name: "AWS", createdAt: "2024-02-01T10:00:00Z" },
  { id: "tag-19", slug: "devops", name: "DevOps", createdAt: "2024-02-02T10:00:00Z" },
  { id: "tag-20", slug: "security", name: "Security", createdAt: "2024-02-03T10:00:00Z" },
];

let MOCK_PROJECTS: AdminProject[] = [
  {
    id: "proj-1",
    slug: "portfolio-site",
    title: "Portfolio Site",
    description: "Personal portfolio with project showcase and blog",
    status: "active",
    githubRepo: "xevion/xevion.dev",
    demoUrl: "https://xevion.dev",
    priority: 100,
    icon: "fa-globe",
    lastGithubActivity: "2024-12-20T15:30:00Z",
    createdAt: "2024-01-10T08:00:00Z",
    updatedAt: "2024-12-20T15:30:00Z",
    tags: [MOCK_TAGS[0], MOCK_TAGS[1], MOCK_TAGS[6], MOCK_TAGS[13]],
  },
  {
    id: "proj-2",
    slug: "task-tracker",
    title: "Task Tracker CLI",
    description: "Command-line task management tool with SQLite backend",
    status: "maintained",
    githubRepo: "xevion/task-tracker",
    demoUrl: null,
    priority: 90,
    icon: "fa-check-square",
    lastGithubActivity: "2024-11-15T10:20:00Z",
    createdAt: "2024-02-05T12:00:00Z",
    updatedAt: "2024-11-15T10:20:00Z",
    tags: [MOCK_TAGS[0], MOCK_TAGS[3], MOCK_TAGS[5]],
  },
  {
    id: "proj-3",
    slug: "api-gateway",
    title: "API Gateway Service",
    description: "High-performance API gateway with rate limiting and caching",
    status: "active",
    githubRepo: "xevion/api-gateway",
    demoUrl: null,
    priority: 85,
    icon: "fa-server",
    lastGithubActivity: "2025-01-05T14:45:00Z",
    createdAt: "2024-03-12T09:30:00Z",
    updatedAt: "2025-01-05T14:45:00Z",
    tags: [MOCK_TAGS[0], MOCK_TAGS[4], MOCK_TAGS[16], MOCK_TAGS[19]],
  },
  {
    id: "proj-4",
    slug: "data-pipeline",
    title: "Data Pipeline Framework",
    description: "ETL framework for processing large datasets",
    status: "archived",
    githubRepo: "xevion/data-pipeline",
    demoUrl: null,
    priority: 50,
    icon: "fa-database",
    lastGithubActivity: "2024-06-10T08:15:00Z",
    createdAt: "2024-01-20T11:00:00Z",
    updatedAt: "2024-06-10T08:15:00Z",
    tags: [MOCK_TAGS[7], MOCK_TAGS[5], MOCK_TAGS[15]],
  },
  {
    id: "proj-5",
    slug: "ml-classifier",
    title: "ML Image Classifier",
    description: "Deep learning model for image classification",
    status: "active",
    githubRepo: "xevion/ml-classifier",
    demoUrl: "https://ml-demo.xevion.dev",
    priority: 80,
    icon: "fa-brain",
    lastGithubActivity: "2024-12-28T16:00:00Z",
    createdAt: "2024-04-01T13:00:00Z",
    updatedAt: "2024-12-28T16:00:00Z",
    tags: [MOCK_TAGS[7], MOCK_TAGS[8], MOCK_TAGS[9]],
  },
  {
    id: "proj-6",
    slug: "container-orchestrator",
    title: "Container Orchestrator",
    description: "Lightweight container orchestration for small deployments",
    status: "active",
    githubRepo: "xevion/orchestrator",
    demoUrl: null,
    priority: 75,
    icon: "fa-ship",
    lastGithubActivity: "2025-01-02T09:30:00Z",
    createdAt: "2024-05-10T10:00:00Z",
    updatedAt: "2025-01-02T09:30:00Z",
    tags: [MOCK_TAGS[0], MOCK_TAGS[9], MOCK_TAGS[10], MOCK_TAGS[18]],
  },
  {
    id: "proj-7",
    slug: "dashboard-components",
    title: "Dashboard Component Library",
    description: "Reusable React components for building admin dashboards",
    status: "maintained",
    githubRepo: "xevion/dashboard-ui",
    demoUrl: "https://dashboard-demo.xevion.dev",
    priority: 70,
    icon: "fa-th-large",
    lastGithubActivity: "2024-10-20T12:00:00Z",
    createdAt: "2024-02-15T14:30:00Z",
    updatedAt: "2024-10-20T12:00:00Z",
    tags: [MOCK_TAGS[1], MOCK_TAGS[11], MOCK_TAGS[13]],
  },
  {
    id: "proj-8",
    slug: "graphql-server",
    title: "GraphQL Server Boilerplate",
    description: "Production-ready GraphQL server with auth and subscriptions",
    status: "active",
    githubRepo: "xevion/graphql-server",
    demoUrl: null,
    priority: 65,
    icon: "fa-project-diagram",
    lastGithubActivity: "2024-12-15T11:30:00Z",
    createdAt: "2024-03-20T08:00:00Z",
    updatedAt: "2024-12-15T11:30:00Z",
    tags: [MOCK_TAGS[1], MOCK_TAGS[4], MOCK_TAGS[14], MOCK_TAGS[15]],
  },
  {
    id: "proj-9",
    slug: "security-scanner",
    title: "Security Scanner",
    description: "Automated security vulnerability scanner for web applications",
    status: "active",
    githubRepo: "xevion/sec-scanner",
    demoUrl: null,
    priority: 60,
    icon: "fa-shield-alt",
    lastGithubActivity: "2024-12-30T10:00:00Z",
    createdAt: "2024-06-01T09:00:00Z",
    updatedAt: "2024-12-30T10:00:00Z",
    tags: [MOCK_TAGS[7], MOCK_TAGS[2], MOCK_TAGS[19]],
  },
  {
    id: "proj-10",
    slug: "cache-optimizer",
    title: "Cache Optimization Library",
    description: "Smart caching layer with automatic invalidation",
    status: "maintained",
    githubRepo: "xevion/cache-lib",
    demoUrl: null,
    priority: 55,
    icon: "fa-bolt",
    lastGithubActivity: "2024-09-10T13:20:00Z",
    createdAt: "2024-04-15T10:30:00Z",
    updatedAt: "2024-09-10T13:20:00Z",
    tags: [MOCK_TAGS[0], MOCK_TAGS[16], MOCK_TAGS[4]],
  },
  {
    id: "proj-11",
    slug: "deployment-tools",
    title: "Deployment Automation Tools",
    description: "CLI tools for automated deployments to multiple cloud providers",
    status: "active",
    githubRepo: "xevion/deploy-tools",
    demoUrl: null,
    priority: 50,
    icon: "fa-rocket",
    lastGithubActivity: "2025-01-01T08:00:00Z",
    createdAt: "2024-07-10T11:00:00Z",
    updatedAt: "2025-01-01T08:00:00Z",
    tags: [MOCK_TAGS[0], MOCK_TAGS[3], MOCK_TAGS[18], MOCK_TAGS[18]],
  },
  {
    id: "proj-12",
    slug: "log-aggregator",
    title: "Log Aggregation Service",
    description: "Centralized logging with search and analytics",
    status: "active",
    githubRepo: "xevion/log-aggregator",
    demoUrl: null,
    priority: 45,
    icon: "fa-file-alt",
    lastGithubActivity: "2024-12-25T15:00:00Z",
    createdAt: "2024-08-05T12:00:00Z",
    updatedAt: "2024-12-25T15:00:00Z",
    tags: [MOCK_TAGS[0], MOCK_TAGS[5], MOCK_TAGS[15]],
  },
  {
    id: "proj-13",
    slug: "ui-playground",
    title: "UI Component Playground",
    description: "Interactive playground for testing UI components",
    status: "maintained",
    githubRepo: "xevion/ui-playground",
    demoUrl: "https://ui.xevion.dev",
    priority: 40,
    icon: "fa-palette",
    lastGithubActivity: "2024-08-20T10:30:00Z",
    createdAt: "2024-05-20T09:00:00Z",
    updatedAt: "2024-08-20T10:30:00Z",
    tags: [MOCK_TAGS[1], MOCK_TAGS[11], MOCK_TAGS[13]],
  },
  {
    id: "proj-14",
    slug: "config-manager",
    title: "Configuration Manager",
    description: "Type-safe configuration management for microservices",
    status: "archived",
    githubRepo: "xevion/config-manager",
    demoUrl: null,
    priority: 30,
    icon: "fa-cog",
    lastGithubActivity: "2024-05-15T14:00:00Z",
    createdAt: "2024-02-28T11:30:00Z",
    updatedAt: "2024-05-15T14:00:00Z",
    tags: [MOCK_TAGS[0], MOCK_TAGS[1]],
  },
  {
    id: "proj-15",
    slug: "websocket-proxy",
    title: "WebSocket Proxy",
    description: "Scalable WebSocket proxy with load balancing",
    status: "active",
    githubRepo: "xevion/ws-proxy",
    demoUrl: null,
    priority: 35,
    icon: "fa-exchange-alt",
    lastGithubActivity: "2024-11-30T16:30:00Z",
    createdAt: "2024-06-15T13:00:00Z",
    updatedAt: "2024-11-30T16:30:00Z",
    tags: [MOCK_TAGS[0], MOCK_TAGS[2], MOCK_TAGS[4]],
  },
];

let MOCK_EVENTS: AdminEvent[] = [
  {
    id: "evt-1",
    timestamp: "2025-01-06T10:30:00Z",
    level: "info",
    target: "project.created",
    message: "Created new project: Portfolio Site",
    metadata: { projectId: "proj-1", userId: "admin" },
  },
  {
    id: "evt-2",
    timestamp: "2025-01-06T09:15:00Z",
    level: "info",
    target: "github.sync",
    message: "GitHub sync completed for 15 projects",
    metadata: { projectCount: 15, duration: 2340 },
  },
  {
    id: "evt-3",
    timestamp: "2025-01-06T08:45:00Z",
    level: "warning",
    target: "github.sync",
    message: "Rate limit approaching: 450/5000 requests remaining",
    metadata: { remaining: 450, limit: 5000 },
  },
  {
    id: "evt-4",
    timestamp: "2025-01-06T08:00:00Z",
    level: "error",
    target: "github.sync",
    message: "Failed to sync project: ml-classifier",
    metadata: { projectId: "proj-5", error: "Repository not found" },
  },
  {
    id: "evt-5",
    timestamp: "2025-01-06T07:30:00Z",
    level: "info",
    target: "tag.created",
    message: "Created new tag: Rust",
    metadata: { tagId: "tag-1" },
  },
  {
    id: "evt-6",
    timestamp: "2025-01-05T23:00:00Z",
    level: "info",
    target: "project.updated",
    message: "Updated project: API Gateway Service",
    metadata: { projectId: "proj-3", changes: ["description", "tags"] },
  },
  {
    id: "evt-7",
    timestamp: "2025-01-05T22:15:00Z",
    level: "info",
    target: "tag.deleted",
    message: "Deleted tag: Legacy",
    metadata: { tagId: "tag-deleted", tagName: "Legacy" },
  },
  {
    id: "evt-8",
    timestamp: "2025-01-05T20:30:00Z",
    level: "error",
    target: "media.upload",
    message: "Failed to upload media: file size exceeds limit",
    metadata: { filename: "banner.png", size: 12582912, limit: 10485760 },
  },
  {
    id: "evt-9",
    timestamp: "2025-01-05T19:00:00Z",
    level: "info",
    target: "project.deleted",
    message: "Deleted project: Old Website",
    metadata: { projectId: "proj-old", projectName: "Old Website" },
  },
  {
    id: "evt-10",
    timestamp: "2025-01-05T18:30:00Z",
    level: "warning",
    target: "cache.invalidation",
    message: "Cache invalidation took longer than expected",
    metadata: { duration: 5420, threshold: 3000 },
  },
];

// Generate additional events for scrolling test
for (let i = 11; i <= 100; i++) {
  const levels: AdminEvent["level"][] = ["info", "warning", "error"];
  const targets = [
    "project.created",
    "project.updated",
    "project.deleted",
    "tag.created",
    "tag.updated",
    "tag.deleted",
    "github.sync",
    "cache.invalidation",
    "media.upload",
  ];

  const level = levels[Math.floor(Math.random() * levels.length)];
  const target = targets[Math.floor(Math.random() * targets.length)];
  const hoursAgo = i;

  const date = new Date();
  date.setHours(date.getHours() - hoursAgo);

  MOCK_EVENTS.push({
    id: `evt-${i}`,
    timestamp: date.toISOString(),
    level,
    target,
    message: `Mock event ${i}: ${target}`,
    metadata: { eventNumber: i },
  });
}

let MOCK_SETTINGS: SiteSettings = {
  identity: {
    displayName: "Ryan Walters",
    occupation: "Full-Stack Software Engineer",
    bio: "A fanatical software engineer with expertise and passion for sound, scalable and high-performance applications. I'm always working on something new.\nSometimes innovative — sometimes crazy.",
    siteTitle: "Xevion.dev",
  },
  socialLinks: [
    {
      id: "social-1",
      platform: "github",
      label: "GitHub",
      value: "https://github.com/Xevion",
      visible: true,
    },
    {
      id: "social-2",
      platform: "linkedin",
      label: "LinkedIn",
      value: "https://linkedin.com/in/ryancwalters",
      visible: true,
    },
    {
      id: "social-3",
      platform: "discord",
      label: "Discord",
      value: "xevion",
      visible: true,
    },
    {
      id: "social-4",
      platform: "email",
      label: "Email",
      value: "your.email@example.com",
      visible: false,
    },
    {
      id: "social-5",
      platform: "pgp",
      label: "PGP Key",
      value: "",
      visible: false,
    },
  ],
  adminPreferences: {
    sessionTimeoutMinutes: 60,
    eventsRetentionDays: 30,
    dashboardDefaultTab: "overview",
  },
};

function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^\w\s-]/g, "")
    .replace(/[\s_-]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

// Admin Projects API
export async function getAdminProjects(): Promise<AdminProject[]> {
  // TODO: Replace with apiFetch('/admin/api/projects') when backend ready
  await new Promise((resolve) => setTimeout(resolve, 100)); // Simulate network delay
  return [...MOCK_PROJECTS].sort((a, b) => b.priority - a.priority);
}

export async function getAdminProject(id: string): Promise<AdminProject | null> {
  // TODO: Replace with apiFetch(`/admin/api/projects/${id}`) when backend ready
  await new Promise((resolve) => setTimeout(resolve, 50));
  return MOCK_PROJECTS.find((p) => p.id === id) || null;
}

export async function createAdminProject(
  data: CreateProjectData,
): Promise<AdminProject> {
  // TODO: Replace with apiFetch('/admin/api/projects', { method: 'POST', body: JSON.stringify(data) })
  await new Promise((resolve) => setTimeout(resolve, 200));

  const now = new Date().toISOString();
  const slug = data.slug || slugify(data.title);
  const tags = MOCK_TAGS.filter((t) => data.tagIds.includes(t.id));

  const newProject: AdminProject = {
    id: generateId(),
    slug,
    title: data.title,
    description: data.description,
    status: data.status,
    githubRepo: data.githubRepo || null,
    demoUrl: data.demoUrl || null,
    priority: data.priority,
    icon: data.icon || null,
    lastGithubActivity: null,
    createdAt: now,
    updatedAt: now,
    tags,
  };

  MOCK_PROJECTS.push(newProject);

  // Add event
  MOCK_EVENTS.unshift({
    id: generateId(),
    timestamp: now,
    level: "info",
    target: "project.created",
    message: `Created new project: ${newProject.title}`,
    metadata: { projectId: newProject.id },
  });

  return newProject;
}

export async function updateAdminProject(
  data: UpdateProjectData,
): Promise<AdminProject> {
  // TODO: Replace with apiFetch(`/admin/api/projects/${data.id}`, { method: 'PUT', body: JSON.stringify(data) })
  await new Promise((resolve) => setTimeout(resolve, 200));

  const index = MOCK_PROJECTS.findIndex((p) => p.id === data.id);
  if (index === -1) throw new Error("Project not found");

  const now = new Date().toISOString();
  const slug = data.slug || slugify(data.title);
  const tags = MOCK_TAGS.filter((t) => data.tagIds.includes(t.id));

  const updatedProject: AdminProject = {
    ...MOCK_PROJECTS[index],
    slug,
    title: data.title,
    description: data.description,
    status: data.status,
    githubRepo: data.githubRepo || null,
    demoUrl: data.demoUrl || null,
    priority: data.priority,
    icon: data.icon || null,
    updatedAt: now,
    tags,
  };

  MOCK_PROJECTS[index] = updatedProject;

  // Add event
  MOCK_EVENTS.unshift({
    id: generateId(),
    timestamp: now,
    level: "info",
    target: "project.updated",
    message: `Updated project: ${updatedProject.title}`,
    metadata: { projectId: updatedProject.id },
  });

  return updatedProject;
}

export async function deleteAdminProject(id: string): Promise<void> {
  // TODO: Replace with apiFetch(`/admin/api/projects/${id}`, { method: 'DELETE' })
  await new Promise((resolve) => setTimeout(resolve, 150));

  const index = MOCK_PROJECTS.findIndex((p) => p.id === id);
  if (index === -1) throw new Error("Project not found");

  const project = MOCK_PROJECTS[index];
  MOCK_PROJECTS.splice(index, 1);

  // Add event
  MOCK_EVENTS.unshift({
    id: generateId(),
    timestamp: new Date().toISOString(),
    level: "info",
    target: "project.deleted",
    message: `Deleted project: ${project.title}`,
    metadata: { projectId: id, projectName: project.title },
  });
}

// Admin Tags API
export async function getAdminTags(): Promise<AdminTagWithCount[]> {
  // TODO: Replace with apiFetch('/admin/api/tags') when backend ready
  await new Promise((resolve) => setTimeout(resolve, 80));

  return MOCK_TAGS.map((tag) => {
    const projectCount = MOCK_PROJECTS.filter((p) =>
      p.tags.some((t) => t.id === tag.id),
    ).length;
    return { ...tag, projectCount };
  }).sort((a, b) => a.name.localeCompare(b.name));
}

export async function createAdminTag(data: CreateTagData): Promise<AdminTag> {
  // TODO: Replace with apiFetch('/admin/api/tags', { method: 'POST', body: JSON.stringify(data) })
  await new Promise((resolve) => setTimeout(resolve, 150));

  const now = new Date().toISOString();
  const slug = data.slug || slugify(data.name);

  const newTag: AdminTag = {
    id: generateId(),
    slug,
    name: data.name,
    createdAt: now,
  };

  MOCK_TAGS.push(newTag);

  // Add event
  MOCK_EVENTS.unshift({
    id: generateId(),
    timestamp: now,
    level: "info",
    target: "tag.created",
    message: `Created new tag: ${newTag.name}`,
    metadata: { tagId: newTag.id },
  });

  return newTag;
}

export async function updateAdminTag(data: UpdateTagData): Promise<AdminTag> {
  // TODO: Replace with apiFetch(`/admin/api/tags/${data.id}`, { method: 'PUT', body: JSON.stringify(data) })
  await new Promise((resolve) => setTimeout(resolve, 150));

  const index = MOCK_TAGS.findIndex((t) => t.id === data.id);
  if (index === -1) throw new Error("Tag not found");

  const slug = data.slug || slugify(data.name);

  const updatedTag: AdminTag = {
    ...MOCK_TAGS[index],
    slug,
    name: data.name,
  };

  MOCK_TAGS[index] = updatedTag;

  // Update tag in all projects
  MOCK_PROJECTS.forEach((project) => {
    const tagIndex = project.tags.findIndex((t) => t.id === data.id);
    if (tagIndex !== -1) {
      project.tags[tagIndex] = updatedTag;
    }
  });

  // Add event
  MOCK_EVENTS.unshift({
    id: generateId(),
    timestamp: new Date().toISOString(),
    level: "info",
    target: "tag.updated",
    message: `Updated tag: ${updatedTag.name}`,
    metadata: { tagId: updatedTag.id },
  });

  return updatedTag;
}

export async function deleteAdminTag(id: string): Promise<void> {
  // TODO: Replace with apiFetch(`/admin/api/tags/${id}`, { method: 'DELETE' })
  await new Promise((resolve) => setTimeout(resolve, 120));

  const index = MOCK_TAGS.findIndex((t) => t.id === id);
  if (index === -1) throw new Error("Tag not found");

  const tag = MOCK_TAGS[index];
  MOCK_TAGS.splice(index, 1);

  // Remove tag from all projects
  MOCK_PROJECTS.forEach((project) => {
    project.tags = project.tags.filter((t) => t.id !== id);
  });

  // Add event
  MOCK_EVENTS.unshift({
    id: generateId(),
    timestamp: new Date().toISOString(),
    level: "info",
    target: "tag.deleted",
    message: `Deleted tag: ${tag.name}`,
    metadata: { tagId: id, tagName: tag.name },
  });
}

// Admin Events API
export async function getAdminEvents(filters?: {
  level?: string;
  target?: string;
  limit?: number;
}): Promise<AdminEvent[]> {
  // TODO: Replace with apiFetch('/admin/api/events?...') when backend ready
  await new Promise((resolve) => setTimeout(resolve, 100));

  let events = [...MOCK_EVENTS];

  if (filters?.level) {
    events = events.filter((e) => e.level === filters.level);
  }

  if (filters?.target) {
    events = events.filter((e) => e.target.includes(filters.target!));
  }

  if (filters?.limit) {
    events = events.slice(0, filters.limit);
  }

  return events;
}

// Admin Stats API
export async function getAdminStats(): Promise<AdminStats> {
  // TODO: Replace with apiFetch('/admin/api/stats') when backend ready
  await new Promise((resolve) => setTimeout(resolve, 80));

  const projectsByStatus: Record<string, number> = {
    active: 0,
    maintained: 0,
    archived: 0,
    hidden: 0,
  };

  MOCK_PROJECTS.forEach((p) => {
    projectsByStatus[p.status]++;
  });

  const today = new Date();
  today.setHours(0, 0, 0, 0);

  const eventsToday = MOCK_EVENTS.filter(
    (e) => new Date(e.timestamp) >= today,
  ).length;

  const errorsToday = MOCK_EVENTS.filter(
    (e) => e.level === "error" && new Date(e.timestamp) >= today,
  ).length;

  return {
    totalProjects: MOCK_PROJECTS.length,
    projectsByStatus: projectsByStatus as Record<
      "active" | "maintained" | "archived" | "hidden",
      number
    >,
    totalTags: MOCK_TAGS.length,
    eventsToday,
    errorsToday,
  };
}

// Settings API
export async function getSettings(): Promise<SiteSettings> {
  // TODO: Replace with apiFetch('/admin/api/settings') when backend ready
  await new Promise((resolve) => setTimeout(resolve, 100));
  return structuredClone(MOCK_SETTINGS);
}

export async function updateSettings(settings: SiteSettings): Promise<SiteSettings> {
  // TODO: Replace with apiFetch('/admin/api/settings', { method: 'PUT', body: JSON.stringify(settings) })
  await new Promise((resolve) => setTimeout(resolve, 200));

  MOCK_SETTINGS = structuredClone(settings);

  // Add event
  MOCK_EVENTS.unshift({
    id: generateId(),
    timestamp: new Date().toISOString(),
    level: "info",
    target: "settings.updated",
    message: "Site settings updated",
    metadata: {},
  });

  return structuredClone(MOCK_SETTINGS);
}
