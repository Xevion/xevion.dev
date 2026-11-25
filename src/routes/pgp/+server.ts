import { pgpKeyResponse } from "#lib/server/static-text.ts";
import type { RequestHandler } from "./$types";

// SvelteKit routes a GET here instead of to +page.svelte unless the client asked for text/html.
export const GET: RequestHandler = () => pgpKeyResponse();
