/**
 * How a session authenticates: an interactive browser cookie, or a long-lived
 * CLI bearer token.
 */
export type SessionType = "browser" | "cli";
