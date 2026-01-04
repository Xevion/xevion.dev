import { redirect } from "@sveltejs/kit";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = async () => {
  // TODO: Fetch resume URL from Rust backend API
  redirect(302, "https://example.com/resume.pdf");
};
