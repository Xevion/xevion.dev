import { PROJECT_STATUSES, type ProjectStatus } from "./types.ts";
import { CliError } from "./errors.ts";

/**
 * Accepts `#6366F1` or `6366F1` and returns the bare lowercase form. The empty
 * string is the "clear this field" sentinel on update commands and passes through.
 */
export function parseHexColor(raw: string): string {
  if (raw === "") return "";
  const hex = raw.startsWith("#") ? raw.slice(1) : raw;
  if (/^[0-9a-fA-F]{6}$/.test(hex)) return hex.toLowerCase();
  throw CliError.invalid(
    `'${raw}' is not a 6-digit hex color (e.g. '6366F1' or '#6366F1')`,
  );
}

export function parseStatus(raw: string): ProjectStatus {
  if ((PROJECT_STATUSES as readonly string[]).includes(raw)) {
    return raw as ProjectStatus;
  }
  throw CliError.invalid(
    `invalid value '${raw}' for '--status': possible values: ${PROJECT_STATUSES.join(", ")}`,
  );
}

export function parseBool(flag: string, raw: string): boolean {
  if (raw === "true") return true;
  if (raw === "false") return false;
  throw CliError.invalid(
    `invalid value '${raw}' for '--${flag}': provided string was not 'true' or 'false'`,
  );
}

export type TagOp = { op: "add" | "remove"; tag: string };

/** Tags for `create`: comma-separated, `+` prefix optional. */
export function parseCreateTags(s: string): string[] {
  return s
    .split(",")
    .map((t) => t.trim())
    .filter((t) => t.length > 0)
    .map((t) => (t.startsWith("+") ? t.slice(1) : t));
}

/** Tags for `update`: a `+` or `-` prefix is required on each. */
export function parseUpdateTags(s: string): TagOp[] {
  return s
    .split(",")
    .map((t) => t.trim())
    .filter((t) => t.length > 0)
    .map((t): TagOp => {
      for (const [prefix, op] of [
        ["+", "add"],
        ["-", "remove"],
      ] as const) {
        if (t.startsWith(prefix)) {
          const tag = t.slice(1);
          if (tag === "") {
            throw CliError.invalid(
              `Tag name cannot be empty after '${prefix}'`,
            );
          }
          return { op, tag };
        }
      }
      throw CliError.invalid(
        `Tag '${t}' requires prefix: use +${t} to add, -${t} to remove`,
      );
    });
}

/** An empty string means "clear"; absent means "keep". */
export function nonEmpty(s: string | undefined): string | undefined {
  return s === undefined || s === "" ? undefined : s;
}
