import { error } from "@sveltejs/kit";
import type { RequestEvent } from "@sveltejs/kit";

/**
 * Check if the request is authenticated
 * Returns the username if authenticated, throws 401 error if not
 */
export function requireAuth(event: RequestEvent): string {
	const sessionUser = event.request.headers.get("x-session-user");

	if (!sessionUser) {
		throw error(401, "Unauthorized");
	}

	return sessionUser;
}

/**
 * Check if the request is authenticated (optional)
 * Returns the username if authenticated, null if not
 */
export function getAuth(event: RequestEvent): string | null {
	return event.request.headers.get("x-session-user");
}
