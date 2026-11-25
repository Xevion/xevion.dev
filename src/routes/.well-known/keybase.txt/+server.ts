import { keybaseResponse } from "#lib/server/static-text.ts";
import type { RequestHandler } from "./$types";

export const GET: RequestHandler = () => keybaseResponse();
