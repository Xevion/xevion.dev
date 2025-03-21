import directus from "@/utils/directus";
import { readSingleton } from "@directus/sdk";
import type { NextApiRequest, NextApiResponse } from "next";

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse,
) {
  if (req.method !== "GET" && req.method !== "HEAD")
    return res.status(405).json({ message: "Method not allowed" });

  // Get the resume
  try {
    const metadata = await directus.request(readSingleton("metadata"));

    const resumeUrl = `${directus.url}assets/${metadata.resume}/${
      metadata.resumeFilename ?? "resume.pdf"
    }`;

    return res.redirect(301, resumeUrl);
  } catch (error) {
    console.error({
      message: "Failed to acquire resume URL",
      error,
    });
    return res.status(500).send({ error: "Failed to acquire resume URL" });
  }
}
