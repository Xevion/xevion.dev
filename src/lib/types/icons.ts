/** Icon identifier in "collection:name" form, e.g. "lucide:home". */
export type IconIdentifier = `${string}:${string}`;

export interface IconCollection {
  id: string;
  name: string;
  total: number;
  category?: string;
}

export interface IconSearchResult {
  identifier: IconIdentifier;
  collection: string;
  name: string;
}

const SEGMENT = /^[a-z0-9-]+$/;

/** Both halves of an identifier are interpolated into storage keys and upstream URLs. */
export function isIconSegment(value: string): boolean {
  return SEGMENT.test(value);
}

export function parseIconIdentifier(
  identifier: string,
): { collection: string; name: string } | null {
  const [collection, name, ...rest] = identifier.split(":");
  if (rest.length > 0 || !collection || !name) return null;
  return isIconSegment(collection) && isIconSegment(name)
    ? { collection, name }
    : null;
}

/** URL of the SVG for an icon on this site. */
export function iconSvgPath(collection: string, name: string): string {
  return `/api/icons/${collection}/${name}.svg`;
}
