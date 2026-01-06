import { readFile } from "fs/promises";
import { join } from "path";
import type { IconifyJSON } from "@iconify/types";
import { getIconData, iconToSVG, replaceIDs } from "@iconify/utils";
import { getLogger } from "@logtape/logtape";
import type {
  IconCollection,
  IconData,
  IconIdentifier,
  IconRenderOptions,
} from "$lib/types/icons";

const logger = getLogger(["server", "icons"]);

// In-memory cache for icon collections
const collectionCache = new Map<string, IconifyJSON>();

// Collections to pre-cache on server startup
const PRE_CACHE_COLLECTIONS = [
  "lucide",
  "simple-icons",
  "material-symbols",
  "heroicons",
  "feather",
];

// Default fallback icon
const DEFAULT_FALLBACK_ICON = "lucide:help-circle";

/**
 * Parse icon identifier into collection and name
 */
function parseIdentifier(
  identifier: string,
): { collection: string; name: string } | null {
  const parts = identifier.split(":");
  if (parts.length !== 2) {
    return null;
  }
  return { collection: parts[0], name: parts[1] };
}

/**
 * Load icon collection from @iconify/json
 */
async function loadCollection(collection: string): Promise<IconifyJSON | null> {
  // Check cache first
  if (collectionCache.has(collection)) {
    return collectionCache.get(collection)!;
  }

  try {
    const iconifyJsonPath = join(
      process.cwd(),
      "node_modules",
      "@iconify",
      "json",
      "json",
      `${collection}.json`,
    );

    const data = await readFile(iconifyJsonPath, "utf-8");
    const iconSet: IconifyJSON = JSON.parse(data);

    // Cache the collection
    collectionCache.set(collection, iconSet);

    logger.debug(`Loaded icon collection: ${collection}`, {
      total: iconSet.info?.total || Object.keys(iconSet.icons).length,
    });

    return iconSet;
  } catch (error) {
    logger.warn(`Failed to load icon collection: ${collection}`, {
      error: error instanceof Error ? error.message : String(error),
    });
    return null;
  }
}

/**
 * Get icon data by identifier
 */
export async function getIcon(identifier: string): Promise<IconData | null> {
  const parsed = parseIdentifier(identifier);
  if (!parsed) {
    logger.warn(`Invalid icon identifier: ${identifier}`);
    return null;
  }

  const { collection, name } = parsed;
  const iconSet = await loadCollection(collection);

  if (!iconSet) {
    return null;
  }

  // Get icon data from the set
  const iconData = getIconData(iconSet, name);
  if (!iconData) {
    logger.warn(`Icon not found: ${identifier}`);
    return null;
  }

  // Build SVG
  const svg = renderIconData(iconData);

  return {
    identifier: identifier as IconIdentifier,
    collection,
    name,
    svg,
  };
}

/**
 * Render icon data to SVG string
 */
function renderIconData(iconData: ReturnType<typeof getIconData>): string {
  if (!iconData) {
    throw new Error("Icon data is null");
  }

  // Convert icon data to SVG attributes
  const renderData = iconToSVG(iconData);

  // Get SVG body
  const body = replaceIDs(iconData.body);

  // Build SVG element
  const attributes = {
    ...renderData.attributes,
    xmlns: "http://www.w3.org/2000/svg",
    "xmlns:xlink": "http://www.w3.org/1999/xlink",
  };

  const attributeString = Object.entries(attributes)
    .map(([key, value]) => `${key}="${value}"`)
    .join(" ");

  return `<svg ${attributeString}>${body}</svg>`;
}

/**
 * Render icon SVG with custom options
 */
export async function renderIconSVG(
  identifier: string,
  options: IconRenderOptions = {},
): Promise<string | null> {
  const iconData = await getIcon(identifier);

  if (!iconData) {
    // Try fallback icon if provided, otherwise use default
    if (identifier !== DEFAULT_FALLBACK_ICON) {
      logger.warn(`Icon not found, using fallback: ${identifier}`);
      return renderIconSVG(DEFAULT_FALLBACK_ICON, options);
    }
    return null;
  }

  let svg = iconData.svg;

  // Apply custom class
  if (options.class) {
    svg = svg.replace("<svg ", `<svg class="${options.class}" `);
  }

  // Apply custom size
  if (options.size) {
    svg = svg.replace(/width="[^"]*"/, `width="${options.size}"`);
    svg = svg.replace(/height="[^"]*"/, `height="${options.size}"`);
  }

  // Apply custom color (replace currentColor)
  if (options.color) {
    svg = svg.replace(/currentColor/g, options.color);
  }

  return svg;
}

/**
 * Get all available collections
 */
export async function getCollections(): Promise<IconCollection[]> {
  const collections: IconCollection[] = [];

  // Load common collections to get metadata
  for (const collectionId of PRE_CACHE_COLLECTIONS) {
    const iconSet = await loadCollection(collectionId);
    if (iconSet && iconSet.info) {
      collections.push({
        id: collectionId,
        name: iconSet.info.name || collectionId,
        total: iconSet.info.total || Object.keys(iconSet.icons).length,
        category: iconSet.info.category,
        prefix: iconSet.prefix,
      });
    }
  }

  return collections;
}

/**
 * Search icons across collections
 */
export async function searchIcons(
  query: string,
  limit: number = 50,
): Promise<{ identifier: string; collection: string; name: string }[]> {
  const results: { identifier: string; collection: string; name: string }[] =
    [];

  // Parse query for collection prefix (e.g., "lucide:home" or "lucide:")
  const colonIndex = query.indexOf(":");
  let targetCollection: string | null = null;
  let searchTerm = query.toLowerCase();

  if (colonIndex !== -1) {
    targetCollection = query.substring(0, colonIndex);
    searchTerm = query.substring(colonIndex + 1).toLowerCase();
  }

  // Determine which collections to search
  const collectionsToSearch = targetCollection
    ? [targetCollection]
    : PRE_CACHE_COLLECTIONS;

  for (const collectionId of collectionsToSearch) {
    if (results.length >= limit) break;

    const iconSet = await loadCollection(collectionId);
    if (!iconSet) continue;

    const iconNames = Object.keys(iconSet.icons);

    for (const iconName of iconNames) {
      if (results.length >= limit) break;

      // Search in icon name
      if (searchTerm === "" || iconName.toLowerCase().includes(searchTerm)) {
        results.push({
          identifier: `${collectionId}:${iconName}`,
          collection: collectionId,
          name: iconName,
        });
      }
    }
  }

  return results;
}

/**
 * Pre-cache common icon collections on server startup
 */
export async function preCacheCollections(): Promise<void> {
  logger.info("Pre-caching icon collections...", {
    collections: PRE_CACHE_COLLECTIONS,
  });

  const promises = PRE_CACHE_COLLECTIONS.map((collection) =>
    loadCollection(collection),
  );
  await Promise.all(promises);

  logger.info("Icon collections pre-cached", {
    cached: collectionCache.size,
  });
}

// TODO: Future enhancement - Support color customization in icon identifiers
// Format idea: "lucide:home#color=blue-500" or separate color field in DB
// Would allow per-project icon theming without hardcoded styles
