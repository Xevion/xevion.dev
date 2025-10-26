import { NextResponse } from "next/server";
import { getPayload } from "payload";
import config from "../../../../payload.config";
import { Octokit } from "@octokit/core";

const octokit = new Octokit({
  auth: process.env.GITHUB_API_TOKEN,
  request: {
    fetch: (url: string | URL, options: RequestInit) => {
      console.log(`${options.method} ${url}`);
      return fetch(url, options);
    },
  },
});

type ProjectResult = {
  id: number;
  previousUpdated: Date | null;
  latestUpdated: Date | null;
};

function getRepository(url: string): [string, string] | null {
  const pattern = /github\.com\/([^/]+)\/([^/]+)/;
  const match = url.match(pattern);

  if (match === null) return null;
  return [match[1]!, match[2]!];
}

function isFulfilled<T>(
  result: PromiseSettledResult<T>,
): result is PromiseFulfilledResult<T> {
  return result.status === "fulfilled";
}

function isRejected<T>(
  result: PromiseSettledResult<T>,
): result is PromiseRejectedResult {
  return result.status === "rejected";
}

async function handleProject({
  id: project_id,
  urls,
  date_updated: previousUpdated,
}: {
  id: number;
  urls: string[];
  date_updated: Date | null;
}): Promise<ProjectResult> {
  const allBranches = await Promise.all(
    urls.map(async (url) => {
      const details = getRepository(url);
      if (!details) {
        return [];
      }

      const [owner, repo] = details;
      const branches = await octokit.request(
        "GET /repos/{owner}/{repo}/branches",
        {
          owner,
          repo,
          headers: {
            "X-GitHub-Api-Version": "2022-11-28",
          },
        },
      );

      return branches.data.map((branch) => ({
        branch: branch.name,
        owner: owner,
        repo: repo,
      }));
    }),
  );

  const latestCommits = allBranches
    .flat()
    .map(async ({ owner, repo, branch }) => {
      const commits = await octokit.request(
        "GET /repos/{owner}/{repo}/commits",
        {
          owner,
          repo,
          sha: branch,
          per_page: 1,
          headers: {
            "X-GitHub-Api-Version": "2022-11-28",
          },
        },
      );
      const latestCommit = commits.data[0];

      if (latestCommit == null) {
        console.warn({
          target: `${owner}/${repo}@${branch}`,
          message: "No commits available",
        });
        return null;
      }

      if (latestCommit.commit.author == null) {
        console.warn({
          target: `${owner}/${repo}@${branch}`,
          sha: latestCommit.sha,
          commit: latestCommit.commit.message,
          url: latestCommit.html_url,
          message: "No author available",
        });
        return null;
      } else if (latestCommit.commit.author.date == null) {
        console.warn({
          target: `${owner}/${repo}@${branch}`,
          sha: latestCommit.sha,
          commit: latestCommit.commit.message,
          url: latestCommit.html_url,
          message: "No date available",
        });
        return null;
      }

      return new Date(latestCommit.commit.author.date);
    });

  const results = await Promise.allSettled(latestCommits);

  results.filter(isRejected).forEach((result) => {
    console.error("Failed to fetch latest commit date", result.reason);
  });

  const latestUpdated = results
    .filter(isFulfilled)
    .map((v) => v.value)
    .filter((v) => v != null)
    .reduce((previous: Date | null, current: Date) => {
      if (previous == null) return current;
      return current > previous ? current : previous;
    }, null);

  if (latestUpdated == null) {
    console.error("Unable to acquire the latest commit date for project");
    return {
      id: project_id,
      previousUpdated,
      latestUpdated: null,
    };
  }

  if (latestUpdated != null && latestUpdated < new Date("2015-01-01")) {
    console.error("Invalid commit date acquired", latestUpdated);
    return {
      id: project_id,
      previousUpdated,
      latestUpdated: null,
    };
  }

  const result = { id: project_id, previousUpdated, latestUpdated: null };

  if (previousUpdated == null || latestUpdated > previousUpdated) {
    const payloadConfig = await config;
    const payload = await getPayload({ config: payloadConfig });

    await payload.update({
      collection: "projects",
      id: project_id,
      data: {
        lastUpdated: latestUpdated.toISOString(),
      },
    });

    return {
      ...result,
      latestUpdated,
    };
  }

  return result;
}

export async function GET(req: Request) {
  const { CRON_SECRET, GITHUB_API_TOKEN } = process.env;
  if (!CRON_SECRET || !GITHUB_API_TOKEN) {
    return NextResponse.json(
      { error: "Missing environment variables" },
      { status: 500 },
    );
  }

  if (process.env.NODE_ENV === "production") {
    const authHeader = req.headers.get("authorization");
    const url = new URL(req.url);
    const secretQueryParam = url.searchParams.get("secret");
    if (
      authHeader !== `Bearer ${CRON_SECRET}` &&
      secretQueryParam !== CRON_SECRET
    ) {
      return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
    }
  }

  let request_count = 0;
  octokit.hook.before("request", async () => {
    request_count++;
  });

  try {
    const payloadConfig = await config;
    const payload = await getPayload({ config: payloadConfig });

    const { docs: projects } = await payload.find({
      collection: "projects",
    });

    const { docs: allLinks } = await payload.find({
      collection: "links",
    });

    const eligibleProjects = projects
      .map((project) => {
        if (!project.autocheckUpdated) return null;

        const urls = allLinks
          .filter((link) => {
            const projectId =
              typeof link.project === "number" ? link.project : link.project.id;
            return projectId === project.id;
          })
          .map((link) => link.url)
          .filter((url) => url.includes("github.com"));

        if (urls.length === 0) return null;

        return {
          id: project.id,
          name: project.name,
          date_updated:
            project.lastUpdated != null ? new Date(project.lastUpdated) : null,
          urls,
        };
      })
      .filter((project) => project !== null);

    const projectPromises = eligibleProjects.map((project) =>
      handleProject({
        id: project.id,
        urls: project.urls,
        date_updated: project.date_updated,
      }),
    );

    const results = await Promise.allSettled(projectPromises);

    const isFailed = results.filter(isRejected).length > results.length * 0.1;

    type Response = {
      request_count: number;
      errors: { project_name: string; reason: string }[];
      ignored: number[];
      changed: { project_name: number; previous: Date | null; latest: Date }[];
    };

    const fulfilled = results.filter(isFulfilled);

    const response: Response = {
      request_count,
      errors: results.filter(isRejected).map((r) => ({
        project_name: "unknown",
        reason: r.reason,
      })),
      ignored: fulfilled
        .filter((r) => r.value.latestUpdated == null)
        .map((r) => r.value.id),
      changed: fulfilled
        .filter((r) => r.value.latestUpdated != null)
        .map((r) => ({
          project_name: r.value.id,
          previous: r.value.previousUpdated,
          latest: r.value.latestUpdated!,
        })),
    };

    return NextResponse.json(response, { status: !isFailed ? 200 : 500 });
  } catch (error) {
    return NextResponse.json({ error }, { status: 500 });
  }
}
