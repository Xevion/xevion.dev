import type { NextApiRequest, NextApiResponse } from "next";
import directus from "@/utils/directus";
import { env } from "@/env/server.mjs";
import { readItems } from "@directus/sdk";

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse,
) {
  const secret = req.headers["authorization"];
  if (typeof secret !== "string" || secret !== env.HEALTHCHECK_SECRET) {
    return res.status(401).json({ error: "Unauthorized" });
  }

  try {
    // Try a simple Directus API call (fetch one project)
    await directus.request(readItems("project", { limit: 1 }));
    return res.status(200).json({ status: "ok" });
  } catch (error) {
    return res
      .status(500)
      .json({ error: "Directus unhealthy", details: String(error) });
  }
}
