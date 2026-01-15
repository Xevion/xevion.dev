import type { IconifyJSON } from "@iconify/types";
import { getIconData, iconToSVG, replaceIDs } from "@iconify/utils";
import { getLogger } from "@logtape/logtape";
import type {
  IconCollection,
  IconIdentifier,
  IconRenderOptions,
} from "$lib/types/icons";

const logger = getLogger(["server", "icons"]);

// In-memory cache for loaded icon collections
const collectionCache = new Map<string, IconifyJSON>();

// Loading promises to prevent concurrent loads of the same collection
const loadingPromises = new Map<string, Promise<IconifyJSON | null>>();

// Collections to pre-cache on server startup
const PRE_CACHE_COLLECTIONS = [
  "lucide",
  "simple-icons",
  "material-symbols",
  "heroicons",
  "feather",
];

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
 * Load icon collection from disk via dynamic import (internal - no caching logic)
 */
async function loadCollectionFromDisk(
  collection: string,
): Promise<IconifyJSON | null> {
  try {
    // Dynamic import - Bun resolves the package path automatically
    const module = await import(`@iconify/json/json/${collection}.json`);
    const iconSet: IconifyJSON = module.default;

    // Cache the collection
    collectionCache.set(collection, iconSet);

    logger.debug("Loaded icon collection", {
      collection,
      total: iconSet.info?.total || Object.keys(iconSet.icons).length,
    });

    return iconSet;
  } catch (error) {
    logger.warn("Failed to load icon collection", {
      collection,
      error: error instanceof Error ? error.message : String(error),
    });
    return null;
  }
}

/**
 * Load icon collection with caching and concurrent load protection.
 * Multiple concurrent requests for the same collection will wait for a single load.
 */
async function loadCollection(collection: string): Promise<IconifyJSON | null> {
  // Return cached if available
  if (collectionCache.has(collection)) {
    return collectionCache.get(collection)!;
  }

  // Wait for in-progress load if another request is already loading this collection
  const existingPromise = loadingPromises.get(collection);
  if (existingPromise) {
    return existingPromise;
  }

  // Start new load and store promise so concurrent requests can wait
  const loadPromise = loadCollectionFromDisk(collection);
  loadingPromises.set(collection, loadPromise);

  try {
    return await loadPromise;
  } finally {
    loadingPromises.delete(collection);
  }
}

/**
 * Render icon data to SVG string (internal)
 */
function renderIconData(
  iconData: ReturnType<typeof getIconData>,
  options: IconRenderOptions = {},
): string {
  if (!iconData) {
    throw new Error("Icon data is null");
  }

  // Convert icon data to SVG attributes
  const renderData = iconToSVG(iconData);

  // Get SVG body
  const body = replaceIDs(iconData.body);

  // Build SVG element with options applied
  const attributes: Record<string, string> = {
    ...renderData.attributes,
    xmlns: "http://www.w3.org/2000/svg",
    "xmlns:xlink": "http://www.w3.org/1999/xlink",
  };

  if (options.class) {
    attributes.class = options.class;
  }
  if (options.size) {
    attributes.width = String(options.size);
    attributes.height = String(options.size);
  }

  const attributeString = Object.entries(attributes)
    .map(([key, value]) => `${key}="${value}"`)
    .join(" ");

  let svg = `<svg ${attributeString}>${body}</svg>`;

  // Apply custom color (replace currentColor)
  if (options.color) {
    svg = svg.replace(/currentColor/g, options.color);
  }

  return svg;
}

/**
 * Get single icon data (for API endpoint use - IconPicker)
 */
export async function getIconForApi(identifier: string): Promise<{
  identifier: string;
  collection: string;
  name: string;
  svg: string;
} | null> {
  const parsed = parseIdentifier(identifier);
  if (!parsed) {
    logger.warn("Invalid icon identifier", { identifier });
    return null;
  }

  const { collection, name } = parsed;
  const iconSet = await loadCollection(collection);

  if (!iconSet) {
    return null;
  }

  const iconData = getIconData(iconSet, name);
  if (!iconData) {
    logger.warn("Icon not found", { identifier });
    return null;
  }

  const svg = renderIconData(iconData);

  return {
    identifier: identifier as IconIdentifier,
    collection,
    name,
    svg,
  };
}

/**
 * Get all available collections with metadata
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
 * Pre-cache common icon collections on server startup.
 * Call this in hooks.server.ts before handling requests.
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
