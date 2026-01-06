/**
 * Single source of truth for all HTTP error codes.
 * Used by:
 * - SvelteKit EntryGenerator (prerendering)
 * - Error page components (rendering)
 * - Rust assets.rs (validation only)
 */
export const ERROR_CODES = {
	// 4xx Client Errors
	400: { message: "Bad request", transient: false },
	401: { message: "Unauthorized", transient: false },
	403: { message: "Forbidden", transient: false },
	404: { message: "Page not found", transient: false },
	405: { message: "Method not allowed", transient: false },
	406: { message: "Not acceptable", transient: false },
	408: { message: "Request timeout", transient: true },
	409: { message: "Conflict", transient: false },
	410: { message: "Gone", transient: false },
	413: { message: "Payload too large", transient: false },
	414: { message: "URI too long", transient: false },
	415: { message: "Unsupported media type", transient: false },
	418: { message: "I'm a teapot", transient: false }, // RFC 2324 Easter egg
	422: { message: "Unprocessable entity", transient: false },
	429: { message: "Too many requests", transient: true },
	451: { message: "Unavailable for legal reasons", transient: false },

	// 5xx Server Errors
	500: { message: "Internal server error", transient: false },
	501: { message: "Not implemented", transient: false },
	502: { message: "Bad gateway", transient: true },
	503: { message: "Service unavailable", transient: true },
	504: { message: "Gateway timeout", transient: true },
	505: { message: "HTTP version not supported", transient: false },
} as const;

export type ErrorCode = keyof typeof ERROR_CODES;

// Helper to check if error code is defined
export function isDefinedErrorCode(code: number): code is ErrorCode {
	return code in ERROR_CODES;
}
