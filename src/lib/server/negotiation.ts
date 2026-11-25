const RAW_USER_AGENT = /^(curl\/|wget\/|httpie\/)|curlie/;

/** True for command-line clients, which want the bare resource instead of the HTML page. */
export function prefersRawContent(
  userAgent: string | null,
  accept: string | null,
): boolean {
  if (userAgent && RAW_USER_AGENT.test(userAgent.toLowerCase())) return true;
  if (!accept) return false;

  const html = accept.indexOf("text/html");
  const wildcard = accept.indexOf("*/*");
  if (html !== -1) return wildcard !== -1 && html > wildcard;
  return wildcard !== -1;
}
