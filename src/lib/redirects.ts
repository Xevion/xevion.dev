/**
 * Accepts a post-login destination only when it is a same-origin absolute path.
 * Rejects protocol-relative `//host`, backslashes and control characters, which browsers may
 * normalise into an off-site URL.
 */
export function safeNextPath(next: string | null | undefined): string | null {
  if (!next || !next.startsWith("/") || next.startsWith("//")) return null;
  // eslint-disable-next-line no-control-regex -- rejecting control characters is the point
  if (/[\\\u0000-\u001f\u007f]/.test(next)) return null;
  if (next.startsWith("/admin/login")) return null;
  return next;
}
