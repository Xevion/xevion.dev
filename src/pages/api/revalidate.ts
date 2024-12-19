import { readItem, readItems } from "@directus/sdk";
import type { NextApiRequest, NextApiResponse } from "next";
import { z } from "zod";
import directus from "@/utils/directus";
import { env } from "@/env/server.mjs";

async function getURLs(
  type: string,
  key: string | number,
  payload: Map<string, unknown>,
): Promise<string[] | null> {
  if (type == "project_link" || type == "project_technology") {
    console.error({
      message: `Failed to provide URls for '${type}' type`,
      type,
      key,
      payload,
    });
    return [];
  }

  if (type === "project") return ["/projects", `/projects/${key}`];
  if (type === "metadata") return ["/"];
  if (type === "technology") {
    const urls = ["/technology"];

    // Get all projects with the technology
    const all_projects = await directus.request(readItems("project"));
    if (all_projects != null) {
      for (const project of all_projects) {
        if (project.technologies?.some((t) => t.id === key))
          urls.push(`/projects/${project.id}`);
      }
    }

    return urls;
  }

  if (type === "projects") {
    const urls = ["/projects", `/projects/${key}`];
    // TODO: If 'featured', index page should be revalidated

    const project = await directus.request(readItem("project", key));
    if (project != null) return urls;
  }

  return null;
}

const requestSchema = z.object({
  type: z.string(),
  keys: z.array(z.string().or(z.number().int())).min(1),
  source: z.map(z.string(), z.any()),
  payload: z.map(z.string(), z.any()),
});

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse,
) {
  if (req.method !== "POST")
    return res.status(405).json({ message: "Method not allowed" });

  if (req.headers["authorization"] !== `Bearer ${env.DIRECTUS_REVALIDATE_KEY}`)
    return res.status(401).json({ message: "Invalid token" });

  try {
    // Verify JSON body
    const { success, data, error } = requestSchema.safeParse(req.body);
    if (!success)
      return res.status(400).json({ message: "Invalid JSON body", error });

    // Get URLs
    const urls = await getURLs(data.type, data.keys[0]!, data.payload);
    if (urls === null)
      return res
        .status(404)
        .json({ revalidated: false, message: "Collection not found" });

    // Revalidate all URLs
    try {
      await Promise.all(urls.map((url) => res.revalidate(url)));
    } catch (error) {
      console.error({ message: "Error while revalidating", error });
      return res.status(500).json({
        revalidated: false,
        message: "Error while revalidating",
        urls,
      });
    }

    // Return success
    return res.json({ revalidated: true, urls });
  } catch (error) {
    console.error({
      message: "Error while preparing to revalidate",
      error,
    });
    return res.status(500).send("Error revalidating");
  }
}
