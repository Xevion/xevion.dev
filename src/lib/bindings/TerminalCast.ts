import type { TerminalLine } from "./TerminalLine";

/**
 * Authored CLI-hero transcript. Stored as JSONB on the project; absent for
 * non-CLI projects (the page then falls through to its normal body).
 */
export type TerminalCast = { prompt: string; lines: Array<TerminalLine> };
