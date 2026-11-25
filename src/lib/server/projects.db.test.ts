import { beforeEach, describe, expect, it } from "vitest";
import { eq } from "drizzle-orm";
import { projectRelations, projects, socialLinks } from "./db/schema.ts";
import { resetDb, testDb } from "./db/test-db.ts";
import {
  addTagToProject,
  getAdminStats,
  getProjectRowByRef,
  getVisibleProjectRow,
  insertProject,
  listProjectCores,
  listSitemapEntries,
  removeTagFromProject,
  replaceProject,
  type ProjectWrite,
} from "./projects.ts";
import { cleanRelatedIds, getRelatedProjects } from "./relations.ts";
import { getSettings, updateSettings } from "./settings.ts";
import {
  createTag,
  getRelatedTags,
  getTagsForProject,
  listTags,
  validateTagInput,
} from "./tags.ts";

const db = testDb();

function write(name: string, over: Partial<ProjectWrite> = {}): ProjectWrite {
  return {
    slug: name.toLowerCase(),
    name,
    shortDescription: "short",
    status: "active",
    hidden: false,
    private: false,
    githubRepo: null,
    githubRepoId: null,
    demoUrl: null,
    projectType: null,
    accentColor: null,
    terminalCast: null,
    detailContent: null,
    renderedHtml: null,
    toc: null,
    ...over,
  };
}

const project = (
  name: string,
  over?: Partial<ProjectWrite>,
  tagIds: string[] = [],
) => insertProject(db, write(name, over), tagIds, []);

const tag = (name: string) => createTag(db, validateTagInput({ name }));

beforeEach(resetDb);

describe("visibility", () => {
  it("keeps hidden projects out of anonymous lists, tag lists and lookups", async () => {
    const t = await tag("rust");
    await project("Shown", {}, [t.id]);
    const hiddenId = await project("Secret", { hidden: true }, [t.id]);

    const anon = await listProjectCores(db, false);
    expect(anon.map((p) => p.slug)).toEqual(["shown"]);
    const anonTag = await listProjectCores(db, false, t.id);
    expect(anonTag.map((p) => p.slug)).toEqual(["shown"]);

    const admin = await listProjectCores(db, true, t.id);
    expect(admin.map((p) => p.slug).sort()).toEqual(["secret", "shown"]);

    expect(await getVisibleProjectRow(db, hiddenId, false)).toBeUndefined();
    expect(await getVisibleProjectRow(db, "secret", false)).toBeUndefined();
    expect(await getVisibleProjectRow(db, "secret", true)).toBeDefined();
  });

  it("counts only visible projects on a tag for anonymous viewers", async () => {
    const t = await tag("rust");
    await project("Shown", {}, [t.id]);
    await project("Secret", { hidden: true }, [t.id]);
    expect((await listTags(db, false))[0]!.projectCount).toBe(1);
    expect((await listTags(db, true))[0]!.projectCount).toBe(2);
  });

  it("omits hidden projects from the sitemap and from anonymous related lists", async () => {
    const shown = await project("Shown");
    const secret = await project("Secret", { hidden: true });
    const host = await insertProject(db, write("Host"), [], [secret, shown]);

    expect((await listSitemapEntries(db)).map((e) => e.slug).sort()).toEqual([
      "host",
      "shown",
    ]);
    expect(
      (await getRelatedProjects(db, host, false)).map((r) => r.slug),
    ).toEqual(["shown"]);
    expect(
      (await getRelatedProjects(db, host, true)).map((r) => r.slug),
    ).toEqual(["secret", "shown"]);
  });
});

describe("ordering", () => {
  it("sorts by GitHub activity, falling back to creation time", async () => {
    const old = await project("Old");
    const active = await project("Active");
    await db
      .update(projects)
      .set({ createdAt: new Date(1_000_000_000_000) })
      .where(eq(projects.id, old));
    await db
      .update(projects)
      .set({
        createdAt: new Date(1_000_000_000_000),
        lastGithubActivity: new Date(1_700_000_000_000),
      })
      .where(eq(projects.id, active));
    expect((await listProjectCores(db, true)).map((p) => p.slug)).toEqual([
      "active",
      "old",
    ]);
  });
});

describe("writes", () => {
  it("replaces tags and the ordered relations in one write", async () => {
    const [a, b] = [await tag("a"), await tag("b")];
    const x = await project("X");
    const y = await project("Y");
    const z = await project("Z");
    const id = await insertProject(db, write("Host"), [a.id], [x, y]);

    await replaceProject(
      db,
      id,
      write("Host"),
      [b.id],
      cleanRelatedIds(id, [z, x, id, z]),
      false,
    );

    expect((await getTagsForProject(db, id)).map((t) => t.slug)).toEqual(["b"]);
    const rels = await db
      .select()
      .from(projectRelations)
      .where(eq(projectRelations.projectId, id));
    expect(
      rels
        .sort((p, q) => p.position - q.position)
        .map((r) => r.relatedProjectId),
    ).toEqual([z, x]);
  });

  it("rejects a duplicate slug", async () => {
    await project("Same");
    await expect(project("Same")).rejects.toThrow(/UNIQUE/i);
  });

  it("resets sync state only when the repo changed", async () => {
    const id = await project("Repo", { githubRepo: "a/b", githubRepoId: 1 });
    await db
      .update(projects)
      .set({
        githubNextSyncAt: new Date(2_000_000_000_000),
        githubErrorCount: 3,
        githubEtag: "e",
      })
      .where(eq(projects.id, id));

    await replaceProject(
      db,
      id,
      write("Repo", { githubRepo: "a/b", githubRepoId: 1 }),
      [],
      [],
      false,
    );
    let row = (await getProjectRowByRef(db, id))!;
    expect(row.githubNextSyncAt).not.toBeNull();
    expect(row.githubEtag).toBe("e");

    await replaceProject(
      db,
      id,
      write("Repo", { githubRepo: "c/d", githubRepoId: 2 }),
      [],
      [],
      true,
    );
    row = (await getProjectRowByRef(db, id))!;
    expect(row.githubNextSyncAt).toBeNull();
    expect(row.githubEtag).toBeNull();
    expect(row.githubErrorCount).toBe(0);
  });

  it("bumps updatedAt when a tag is attached or detached", async () => {
    const t = await tag("rust");
    const id = await project("P");
    const stale = new Date(1_000_000_000_000);
    const updatedAt = async () =>
      (await getProjectRowByRef(db, id))!.updatedAt.getTime();

    await db
      .update(projects)
      .set({ updatedAt: stale })
      .where(eq(projects.id, id));
    await addTagToProject(db, id, t.id);
    expect(await updatedAt()).toBeGreaterThan(stale.getTime());

    await db
      .update(projects)
      .set({ updatedAt: stale })
      .where(eq(projects.id, id));
    await removeTagFromProject(db, id, t.id);
    expect(await updatedAt()).toBeGreaterThan(stale.getTime());
  });
});

describe("tags", () => {
  it("ranks related tags by shared projects and respects visibility", async () => {
    const [a, b, c] = [await tag("a"), await tag("b"), await tag("c")];
    await project("P1", {}, [a.id, b.id]);
    await project("P2", {}, [a.id, b.id, c.id]);
    await project("P3", { hidden: true }, [a.id, c.id]);

    const admin = await getRelatedTags(db, a.id, true);
    expect(admin.map((r) => [r.slug, r.cooccurrenceCount])).toEqual([
      ["b", 2],
      ["c", 2],
    ]);
    const anon = await getRelatedTags(db, a.id, false);
    expect(anon.map((r) => [r.slug, r.cooccurrenceCount])).toEqual([
      ["b", 2],
      ["c", 1],
    ]);
  });

  it("rejects duplicate names case-insensitively", async () => {
    await tag("Rust");
    await expect(tag("rust")).rejects.toMatchObject({ status: 409 });
  });
});

describe("stats", () => {
  it("always reports every status", async () => {
    await project("A", { status: "archived" });
    const stats = await getAdminStats(db);
    expect(stats.projectsByStatus).toEqual({
      active: 0,
      maintained: 0,
      archived: 1,
    });
    expect(stats.totalProjects).toBe(1);
  });
});

describe("settings", () => {
  it("updates social links by id and rejects unknown ids", async () => {
    const [link] = await db
      .insert(socialLinks)
      .values({
        platform: "github",
        label: "GH",
        value: "x",
        icon: "i",
        displayOrder: 0,
      })
      .returning();
    const identity = {
      displayName: "N",
      occupation: "O",
      bio: "B",
      siteTitle: "T",
    };

    const out = await updateSettings(
      db,
      {
        identity,
        socialLinks: [
          { ...link!, label: "GitHub", visible: false, displayOrder: 5 },
        ],
      },
      "https://m.test",
    );
    expect(out.identity.displayName).toBe("N");
    expect(out.socialLinks[0]).toMatchObject({
      label: "GitHub",
      visible: false,
      displayOrder: 5,
    });

    await expect(
      updateSettings(
        db,
        { identity, socialLinks: [{ ...link!, id: crypto.randomUUID() }] },
        "https://m.test",
      ),
    ).rejects.toMatchObject({ status: 404 });
    expect(
      (await getSettings(db, "https://m.test")).identity.resumeUrl,
    ).toBeNull();
  });
});
