/**
 * Telemetry client wrapper for PostHog with type-safe event tracking.
 * Provides console logging in development when PostHog is not configured.
 */

import posthog from "posthog-js";
import { browser, dev } from "$app/environment";
import { env } from "$env/dynamic/public";
import type { TelemetryEvent, ExternalLinkEvent, ErrorEvent } from "./events";

// Environment variables for PostHog configuration
// Set PUBLIC_POSTHOG_KEY and PUBLIC_POSTHOG_HOST in your .env file
// Using dynamic/public so they can be set at runtime (not just build time)
const POSTHOG_KEY = env.PUBLIC_POSTHOG_KEY;
const POSTHOG_HOST = env.PUBLIC_POSTHOG_HOST;

class TelemetryClient {
  private initialized = false;
  private enabled = false;

  /**
   * Centralized logging method that only logs in development mode
   */
  private log(message: string, data?: unknown): void {
    if (dev) {
      if (data !== undefined) {
        console.log(`[Telemetry] ${message}`, data);
      } else {
        console.log(`[Telemetry] ${message}`);
      }
    }
  }

  /**
   * Initialize the PostHog client if keys are available
   */
  init(): void {
    if (this.initialized || !browser) return;

    // Only enable PostHog if both key and host are configured
    if (POSTHOG_KEY && POSTHOG_HOST) {
      posthog.init(POSTHOG_KEY, {
        api_host: POSTHOG_HOST,
        ui_host: "https://us.posthog.com", // For toolbar links
        capture_pageview: false, // We handle page views manually
        capture_pageleave: true,
        autocapture: true,
        persistence: "localStorage",
        // Session replay config
        session_recording: {
          recordCrossOriginIframes: true,
        },
      });

      this.enabled = true;
      this.log("PostHog initialized");

      if (dev) {
        posthog.debug();
      }
    } else {
      this.enabled = false;
      this.log(
        "PostHog not configured (missing PUBLIC_POSTHOG_KEY or PUBLIC_POSTHOG_HOST)",
      );
    }

    this.initialized = true;
  }

  /**
   * Track a telemetry event with type safety
   */
  track<E extends TelemetryEvent>(event: E): void {
    if (!browser) return;

    this.log(event.name, event.properties);

    if (this.enabled) {
      posthog.capture(event.name, event.properties);
    }
  }

  /**
   * Convenience method for tracking page views
   */
  trackPageView(route: string): void {
    this.track({
      name: "page_view",
      properties: {
        route,
        referrer: browser ? document.referrer : undefined,
      },
    });
  }

  /**
   * Identify a user with properties
   */
  identify(userId: string, properties?: Record<string, unknown>): void {
    if (!browser) return;

    this.log("identify", { userId, properties });

    if (this.enabled) {
      posthog.identify(userId, properties);
    }
  }

  /**
   * Reset user identification (e.g., on logout)
   */
  reset(): void {
    if (!this.initialized || !browser) return;

    this.log("reset");

    if (this.enabled) {
      posthog.reset();
    }
  }

  /**
   * Check if telemetry is enabled
   */
  isEnabled(): boolean {
    return this.enabled;
  }

  /**
   * Identify an admin user with admin flag
   */
  identifyAdmin(username: string): void {
    this.identify(username, {
      is_admin: true,
      admin_username: username,
    });
  }

  /**
   * Track an external link click
   */
  trackExternalLink(
    url: string,
    context: ExternalLinkEvent["properties"]["context"],
  ): void {
    this.track({
      name: "external_link_click",
      properties: { url, context },
    });
  }

  /**
   * Track an error event
   */
  trackError(
    errorType: ErrorEvent["properties"]["errorType"],
    message: string,
    options?: { stack?: string; context?: Record<string, unknown> },
  ): void {
    this.track({
      name: "error",
      properties: {
        errorType,
        message,
        stack: options?.stack,
        context: options?.context,
      },
    });
  }
}

/**
 * Singleton telemetry client instance
 */
export const telemetry = new TelemetryClient();
