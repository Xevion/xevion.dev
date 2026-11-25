import { describe, expect, it } from "vitest";
import {
  buildLinks,
  toApiAdminProject,
  type ProjectCore,
} from "./project-mapping.ts";

const core: ProjectCore = {
  id: "id",
  slug: "slug",
  name: "Name",
  shortDescription: "Short",
  status: "active",
  hidden: false,
  private: false,
  githubRepo: "xevion/example",
  demoUrl: "https://example.dev",
  projectType: null,
  accentColor: null,
  lastGithubActivity: null,
  githubSyncedAt: null,
  githubSyncError: null,
  createdAt: new Date("2025-01-02T03:04:05Z"),
};

describe("buildLinks", () => {
  it("links the repo and the demo", () => {
    expect(buildLinks(core)).toEqual([
      { url: "https://github.com/xevion/example", title: "GitHub" },
      { url: "https://example.dev", title: "Demo" },
    ]);
  });

  it("suppresses the repo link of a private project", () => {
    expect(buildLinks({ ...core, private: true })).toEqual([
      { url: "https://example.dev", title: "Demo" },
    ]);
  });

  it("is empty with neither repo nor demo", () => {
    expect(buildLinks({ ...core, githubRepo: null, demoUrl: null })).toEqual(
      [],
    );
  });
});

describe("toApiAdminProject", () => {
  it("falls back to createdAt for lastActivity and omits unset optionals", () => {
    const out = toApiAdminProject(core, [], []);
    expect(out.lastActivity).toBe("2025-01-02T03:04:05.000Z");
    expect(out).not.toHaveProperty("accentColor");
    expect(out).not.toHaveProperty("githubSyncError");
  });

  it("prefers the synced GitHub activity", () => {
    const out = toApiAdminProject(
      { ...core, lastGithubActivity: new Date("2026-03-04T00:00:00Z") },
      [],
      [],
    );
    expect(out.lastActivity).toBe("2026-03-04T00:00:00.000Z");
  });
});
