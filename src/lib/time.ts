// Shared time formatting. Every "X ago" rendering in the app routes through
// here so the buckets can't drift between the public cards, the event log, and
// the admin tables.

/**
 * Compact relative age: "just now", "5m ago", "3h ago", "yesterday", "9d ago",
 * "2mo ago", "2y ago".
 *
 * `now` is injectable so SSR and client hydration agree on the bucket — calling
 * `Date.now()` per render can otherwise flip the result between passes.
 */
export function timeAgo(dateString: string, now: number = Date.now()): string {
  const diffMs = now - new Date(dateString).getTime();
  const diffMins = Math.floor(diffMs / 60_000);
  const diffHours = Math.floor(diffMs / 3_600_000);
  const diffDays = Math.floor(diffMs / 86_400_000);

  if (diffMins < 1) return "just now";
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffHours <= 48) return "yesterday";
  if (diffDays < 30) return `${diffDays}d ago`;
  if (diffDays < 365) return `${Math.floor(diffDays / 30)}mo ago`;
  return `${Math.floor(diffDays / 365)}y ago`;
}

/**
 * Relative age anchored to a concrete time, so a value isn't purely fuzzy:
 * "40m ago at 2:09 PM" for today, "9d ago on Mar 5" otherwise. Use where a
 * single bare "X ago" would leave the reader guessing the actual moment.
 */
export function timeAgoExact(
  dateString: string,
  now: number = Date.now(),
): string {
  const date = new Date(dateString);
  const ago = timeAgo(dateString, now);
  const sameDay = new Date(now).toDateString() === date.toDateString();
  const anchor = sameDay
    ? `at ${date.toLocaleTimeString("en-US", { timeStyle: "short" })}`
    : `on ${date.toLocaleDateString("en-US", { month: "short", day: "numeric" })}`;
  return `${ago} ${anchor}`;
}

/** Absolute local timestamp for hover titles, e.g. "Mar 5, 2026, 3:04 PM". */
export function formatDateTime(dateString: string): string {
  return new Date(dateString).toLocaleString("en-US", {
    dateStyle: "medium",
    timeStyle: "short",
  });
}
