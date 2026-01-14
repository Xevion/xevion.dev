/**
 * Type-safe telemetry event system using discriminated unions.
 * All events must have a 'name' discriminator property.
 */

/**
 * Page view tracking event
 */
export type PageViewEvent = {
  name: "page_view";
  properties: {
    route: string;
    referrer?: string;
  };
};

/**
 * Project card interaction event
 */
export type ProjectInteractionEvent = {
  name: "project_interaction";
  properties: {
    action: "card_view" | "github_click" | "demo_click";
    projectSlug: string;
    projectName: string;
    targetUrl?: string;
  };
};

/**
 * Tag interaction event (for fuzzy discovery feature)
 */
export type TagInteractionEvent = {
  name: "tag_interaction";
  properties: {
    action: "select" | "deselect" | "reset";
    tagSlug?: string;
    selectedTags: string[];
  };
};

/**
 * External link click event
 */
export type ExternalLinkEvent = {
  name: "external_link_click";
  properties: {
    url: string;
    context: "social" | "project" | "footer" | "pgp" | "resume";
  };
};

/**
 * Theme preference change event
 */
export type ThemeEvent = {
  name: "theme_change";
  properties: {
    theme: "light" | "dark";
  };
};

/**
 * Error tracking event
 */
export type ErrorEvent = {
  name: "error";
  properties: {
    errorType: "network_error" | "validation_error" | "runtime_error" | string;
    message: string;
    stack?: string;
    context?: Record<string, unknown>;
  };
};

/**
 * PGP page interaction event
 */
export type PgpInteractionEvent = {
  name: "pgp_interaction";
  properties: {
    action: "copy_key" | "download_key" | "copy_command";
  };
};

/**
 * Discriminated union of all possible events
 */
export type TelemetryEvent =
  | PageViewEvent
  | ProjectInteractionEvent
  | TagInteractionEvent
  | ExternalLinkEvent
  | ThemeEvent
  | ErrorEvent
  | PgpInteractionEvent;

/**
 * Helper type to extract event properties by event name
 */
export type EventProperties<T extends TelemetryEvent["name"]> = Extract<
  TelemetryEvent,
  { name: T }
>["properties"];
