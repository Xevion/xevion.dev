import { ERROR_CODES } from "$lib/error-codes";
import type { EntryGenerator, PageServerLoad } from "./$types";

/**
 * Tell SvelteKit to prerender all defined error codes.
 * This generates static HTML files at build time.
 */
export const entries: EntryGenerator = () => {
	return Object.keys(ERROR_CODES).map((code) => ({ code }));
};

export const prerender = true;

/**
 * Load error metadata for the page.
 * This runs during prerendering to generate static HTML.
 */
export const load: PageServerLoad = ({ params }) => {
	const code = parseInt(params.code, 10) as keyof typeof ERROR_CODES;

	return {
		code,
		...ERROR_CODES[code],
	};
};
