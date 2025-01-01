/**
 * This is a cron job handler for acquiring the latest 'updated' data for the site's projects.
 *
 * 1) Fetch the list of all projects including their link URLs.
 * 2) Filter the list only for projects with 'autocheck_update' enabled and any 'github.com' link.
 * 3) For each project, query the GitHub API for the latest commit date on all branches.
 * 4) If the latest commit date is newer than the project's 'last_updated' date, update the project's 'last_updated' date.
 * 5) If any project's 'last_updated' date was updated, revalidate the index and/or project page.
 * 6) Report the results of this cron job invocation.
 *
 * - This cron job runs at least once a day, at most once an hour.
 * - This cron job is completely asynchronous but respects GitHub API rate limits.
 * - This cron job requires authentication with the Directus API.
 * - This cron job requires authentication with the GitHub API (mostly for rate limits).
 */
import directus, { ProjectLink } from "@/utils/directus";
import { readItems, updateItem } from "@directus/sdk";
import { NextApiRequest, NextApiResponse } from "next";
import { Octokit } from "@octokit/core";
import { isFulfilled, isRejected } from "@/utils/types";

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
  id: string;
  previousUpdated: Date | null;
  latestUpdated: Date | null;
};

function getRepository(url: string): [string, string] | null {
  const pattern = /github.com\/([^/]+)\/([^/]+)/;
  const match = pattern.exec(url);

  if (match === null) return null;
  return [match[1]!, match[2]!];
}

async function handleProject({
  id: project_id,
  urls,
  date_updated: previousUpdated,
}: {
  id: string;
  urls: string[];
  date_updated: Date | null;
}): Promise<ProjectResult> {
  // Extract the branches from each URL
  const allBranches = await Promise.all(
    urls.map(async (url) => {
      const details = getRepository(url);
      if (!details) {
        return [];
      }

      // TODO: Handle deduplication of repository targets
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

  // Get the latest commit date for each branch (flattened)
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

      // Commits not returned
      if (latestCommit == null) {
        console.warn({
          target: `${owner}/${repo}@${branch}`,
          message: "No commits available",
        });
        return null;
      }

      // Handle missing commit data in unpredictable cases
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

  // Handle the promises that failed
  results.filter(isRejected).forEach((result) => {
    // TODO: Add more context to the error message
    console.error("Failed to fetch latest commit date", result.reason);
  });

  // Find the latest commit date
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

  // Ensure it's a reasonable date
  if (latestUpdated != null && latestUpdated < new Date("2015-01-01")) {
    console.error("Invalid commit date acquired", latestUpdated);
    return {
      id: project_id,
      previousUpdated,
      latestUpdated: null,
    };
  }

  const result = { id: project_id, previousUpdated, latestUpdated: null };

  // Update the project's 'last_updated' date if the latest commit date is newer
  if (previousUpdated == null || latestUpdated > previousUpdated) {
    await directus.request(
      updateItem("project", project_id, {
        date_updated: latestUpdated,
      }),
    );

    // 'latestUpdated' is not null ONLY if the project was actually updated
    return {
      ...result,
      latestUpdated,
    };
  }

  return result;
}

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse,
) {
  // Check for the required environment variables
  const { CRON_SECRET, GITHUB_API_TOKEN, DIRECTUS_API_TOKEN } = process.env;
  if (!CRON_SECRET || !GITHUB_API_TOKEN || !DIRECTUS_API_TOKEN) {
    res.status(500).json({ error: "Missing environment variables" });
  }

  // Ensure the cron request is authenticated
  if (process.env.NODE_ENV !== "development") {
    const authHeader = req.headers["authorization"];
    if (authHeader !== `Bearer ${CRON_SECRET}`) {
      return new Response("Unauthorized", {
        status: 401,
      });
    }
  }

  let request_count = 0;
  octokit.hook.before("request", async () => {
    request_count++;
  });

  try {
    // Fetch the list of all projects including their link URLs.
    const projects = await directus.request(
      readItems("project", {
        fields: [
          "id",
          "name",
          "autocheckUpdated",
          "date_updated",
          { links: ["url"] },
        ],
      }),
    );

    // Filter the list only for projects with 'autocheck_update' enabled and any 'github.com' link.
    const eligibleProjects = projects
      .map((project) => {
        // Skip projects that don't have autocheckUpdated enabled.
        if (!project.autocheckUpdated) return null;

        // Acquire the URL from the link, then filter out any non-GitHub URLs.
        const urls = project
          .links!.map((link) => {
            return (<ProjectLink>link).url;
          })
          .filter((url) => url.includes("github.com"));

        // Skip projects that don't have any GitHub URLs.
        if (urls.length === 0) return null;

        // Return the project's most important data for further processing.
        return {
          id: project.id,
          name: project.name,
          date_updated: project.date_updated,
          urls,
        };
      })
      // null values are still included in the array, so filter them out.
      .filter((project) => project !== null);

    // Log the date_updated for each project
    eligibleProjects.forEach((project) => {
      console.log({
        name: project.name,
        date_updated: project.date_updated,
      });
    });

    // For each project, query the GitHub API for the latest commit date on all branches.
    const projectPromises = eligibleProjects.map((project) =>
      handleProject({
        id: project.id,
        urls: project.urls,
        date_updated:
          project.date_updated != null ? new Date(project.date_updated) : null,
      }),
    );

    // Wait for all project promises to resolve
    const results = await Promise.allSettled(projectPromises);

    // If more than 10% of the requests failed, return an error status code
    const isFailed = results.filter(isRejected).length > results.length * 0.1;

    type Response = {
      request_count: number;
      errors: { project_name: string; reason: string }[];
      ignored: string[];
      changed: { project_name: string; previous: Date | null; latest: Date }[];
    };

    const fulfilled = results.filter(isFulfilled);

    const response: Response = {
      request_count,
      errors: results.filter(isRejected).map((r) => ({
        // TODO: Fix this project name
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

    res.status(!isFailed ? 200 : 500).json(response);
  } catch (error) {
    res.status(500).json({ error });
    return;
  }
}
