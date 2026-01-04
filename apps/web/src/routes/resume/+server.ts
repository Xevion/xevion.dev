import { redirect } from "@sveltejs/kit";
import { getDb } from "$lib/server/db";
import { metadata } from "@xevion/db";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async (event) => {
  const db = getDb(event);

  // TODO: Query the metadata global for resume URL
  // For now, redirect to a placeholder
  // const meta = await db.select().from(metadata).limit(1);

  // Placeholder redirect until we have the schema set up
  redirect(302, "https://example.com/resume.pdf");
};
