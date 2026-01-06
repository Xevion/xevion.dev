/**
 * Icon identifier in format "collection:name"
 * Example: "lucide:home", "simple-icons:react"
 */
export type IconIdentifier = `${string}:${string}`;

/**
 * Icon metadata for search results and picker
 */
export interface IconMetadata {
	identifier: IconIdentifier;
	collection: string;
	name: string;
	keywords?: string[];
}

/**
 * Icon collection information
 */
export interface IconCollection {
	id: string;
	name: string;
	total: number;
	category?: string;
	prefix: string;
}

/**
 * Full icon data with SVG
 */
export interface IconData {
	identifier: IconIdentifier;
	collection: string;
	name: string;
	svg: string;
}

/**
 * Options for rendering icon SVG
 */
export interface IconRenderOptions {
	class?: string;
	size?: number;
	color?: string;
}
