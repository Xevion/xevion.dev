import { describe, expect, it } from "vitest";
import {
  MAX_INTERVAL_MS,
  MIN_INTERVAL_MS,
  calculateCheckIntervalMs,
  errorBackoffMs,
  isRateLimited,
  mergeActivity,
  normalizeGithubRepo,
  parseGithubRepo,
  rateLimitBackoffMs,
} from "./github.ts";

const DAY = 24 * 3600_000;
const NOW = Date.UTC(2026, 5, 1);
const daysAgo = (n: number) => new Date(NOW - n * DAY);

describe("normalizeGithubRepo", () => {
  it.each([
    ["Xevion/xevion.dev", "Xevion/xevion.dev"],
    ["https://github.com/Xevion/xevion.dev", "Xevion/xevion.dev"],
    ["http://github.com/owner/repo/", "owner/repo"],
    ["github.com/owner/repo.git", "owner/repo"],
    ["  owner/repo  ", "owner/repo"],
    ["my-org/my_repo.rs", "my-org/my_repo.rs"],
  ])("accepts %s", (input, expected) => {
    expect(normalizeGithubRepo(input)).toBe(expected);
  });

  it.each([
    "not-a-repo",
    "owner/repo/extra",
    "owner/",
    "/repo",
    "ow ner/repo",
    "owner/re po",
    "own_er/repo",
    `${"a".repeat(40)}/repo`,
    "",
  ])("rejects %j", (input) => {
    expect(normalizeGithubRepo(input)).toBeNull();
  });

  it("allows a 39 character owner", () => {
    expect(normalizeGithubRepo(`${"a".repeat(39)}/r`)).toBe(
      `${"a".repeat(39)}/r`,
    );
  });
});

describe("parseGithubRepo", () => {
  it("splits only the canonical two-segment form", () => {
    expect(parseGithubRepo("owner/repo")).toEqual({
      owner: "owner",
      repo: "repo",
    });
    expect(parseGithubRepo("owner")).toBeNull();
    expect(parseGithubRepo("owner/repo/extra")).toBeNull();
    expect(parseGithubRepo("/repo")).toBeNull();
  });
});

describe("calculateCheckIntervalMs", () => {
  it("uses the minimum for same-day activity and the maximum with none", () => {
    expect(calculateCheckIntervalMs(daysAgo(0), NOW)).toBe(MIN_INTERVAL_MS);
    expect(calculateCheckIntervalMs(null, NOW)).toBe(MAX_INTERVAL_MS);
  });

  it("clamps activity older than 90 days to the maximum", () => {
    expect(calculateCheckIntervalMs(daysAgo(200), NOW)).toBe(MAX_INTERVAL_MS);
  });

  it("treats future timestamps as today", () => {
    expect(calculateCheckIntervalMs(daysAgo(-3), NOW)).toBe(MIN_INTERVAL_MS);
  });

  it("follows the log curve and grows with age", () => {
    const minutes = (days: number) =>
      calculateCheckIntervalMs(daysAgo(days), NOW) / 60_000;
    expect(Math.round(minutes(1))).toBe(234);
    expect(Math.round(minutes(7))).toBe(672);
    expect(Math.round(minutes(30))).toBe(1100);
    expect(minutes(1)).toBeLessThan(minutes(7));
    expect(minutes(7)).toBeLessThan(minutes(30));
  });
});

describe("errorBackoffMs", () => {
  const fresh = daysAgo(0);

  it("doubles per consecutive error", () => {
    expect(errorBackoffMs(fresh, 1, NOW)).toBe(MIN_INTERVAL_MS * 2);
    expect(errorBackoffMs(fresh, 2, NOW)).toBe(MIN_INTERVAL_MS * 4);
    expect(errorBackoffMs(fresh, 3, NOW)).toBe(MIN_INTERVAL_MS * 8);
  });

  it("stops doubling at 2^6", () => {
    expect(errorBackoffMs(fresh, 6, NOW)).toBe(MIN_INTERVAL_MS * 64);
    expect(errorBackoffMs(fresh, 50, NOW)).toBe(MIN_INTERVAL_MS * 64);
  });

  it("never waits longer than the maximum interval", () => {
    expect(errorBackoffMs(null, 1, NOW)).toBe(MAX_INTERVAL_MS);
    expect(errorBackoffMs(daysAgo(30), 2, NOW)).toBe(MAX_INTERVAL_MS);
  });
});

describe("mergeActivity", () => {
  const early = new Date("2024-01-01T00:00:00Z");
  const late = new Date("2024-06-01T00:00:00Z");

  it("takes the latest and treats null as no signal", () => {
    expect(mergeActivity(early, late)).toBe(late);
    expect(mergeActivity(late, early)).toBe(late);
    expect(mergeActivity(early, null)).toBe(early);
    expect(mergeActivity(null, late)).toBe(late);
    expect(mergeActivity(null, null)).toBeNull();
  });
});

describe("rate limit detection", () => {
  it("flags 429 and 403 with an exhausted quota only", () => {
    expect(isRateLimited(429, null)).toBe(true);
    expect(isRateLimited(403, "0")).toBe(true);
    expect(isRateLimited(403, "57")).toBe(false);
    expect(isRateLimited(403, null)).toBe(false);
    expect(isRateLimited(404, "0")).toBe(false);
  });

  it("prefers retry-after, then the reset time, clamping the past to zero", () => {
    expect(rateLimitBackoffMs("120", "9999999999", 0)).toBe(120_000);
    expect(rateLimitBackoffMs(null, "1000", 700)).toBe(300_000);
    expect(rateLimitBackoffMs(null, "500", 900)).toBe(0);
    expect(rateLimitBackoffMs(null, null, 0)).toBeNull();
    expect(rateLimitBackoffMs("soon", null, 0)).toBeNull();
  });
});
