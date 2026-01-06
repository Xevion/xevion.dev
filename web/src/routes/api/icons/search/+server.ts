import { json } from "@sveltejs/kit";
import type { RequestHandler } from "./$types";
import { requireAuth } from "$lib/server/auth";
import { searchIcons } from "$lib/server/icons";

export const GET: RequestHandler = async (event) => {
	// Require authentication
	requireAuth(event);

	const query = event.url.searchParams.get("q") || "";
	const limitParam = event.url.searchParams.get("limit");
	const limit = limitParam ? parseInt(limitParam, 10) : 50;

	const results = await searchIcons(query, limit);

	return json({
		icons: results,
		query,
		count: results.length,
	});
};
