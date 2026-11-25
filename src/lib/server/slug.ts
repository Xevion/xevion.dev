const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const HEX6 = /^[0-9a-f]{6}$/i;

export const isUuid = (value: string): boolean => UUID.test(value);

/** Six hex digits with no `#` and no alpha, the stored form of every color. */
export const isHexColor = (value: string): boolean => HEX6.test(value);

/** Accepts `#6366F1` or `6366F1` and returns the bare lowercase form, or null when malformed. */
export function normalizeHexColor(value: string): string | null {
  const bare = value.startsWith("#") ? value.slice(1) : value;
  return isHexColor(bare) ? bare.toLowerCase() : null;
}

/** Lowercase, keeps letters and digits, turns whitespace and `-_.` into single hyphens. */
export function slugify(text: string): string {
  let out = "";
  for (const ch of text.toLowerCase()) {
    if (/[\p{L}\p{N}]/u.test(ch)) out += ch;
    else if (/\s/u.test(ch) || ch === "-" || ch === "_" || ch === ".")
      out += "-";
  }
  return out
    .split("-")
    .filter((part) => part.length > 0)
    .join("-");
}
