import { json } from "@sveltejs/kit";
import type { RequestHandler } from "./$types";
import { requireAuth } from "$lib/server/auth";
import { getCollections } from "$lib/server/icons";

export const GET: RequestHandler = async (event) => {
	// Require authentication
	requireAuth(event);

	const collections = await getCollections();

	return json({
		collections,
		count: collections.length,
	});
};
