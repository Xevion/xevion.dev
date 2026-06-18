import type { ApiSiteIdentity, ApiSocialLink } from "$lib/bindings";

/**
 * Build the site-wide `Person` + `WebSite` JSON-LD `<script>` element for the
 * homepage head.
 *
 * `sameAs` only carries platforms with a canonical public profile URL
 * (`github`/`linkedin`, whose `value` is already a full URL); email becomes
 * `Person.email`, and platforms without a stable URL (e.g. Discord) are omitted.
 * `<` is escaped in the JSON payload and the closing tag is split, so the
 * serialized data can't break out of the injected `<script>` element.
 */
export function homepageJsonLdScript(
  identity: ApiSiteIdentity,
  socialLinks: ApiSocialLink[],
  origin: string,
): string {
  const visible = socialLinks.filter((link) => link.visible);

  const sameAs = visible
    .filter(
      (link) => link.platform === "github" || link.platform === "linkedin",
    )
    .map((link) => link.value);

  const email = visible.find((link) => link.platform === "email")?.value;

  const person = {
    "@context": "https://schema.org",
    "@type": "Person",
    name: identity.displayName,
    url: origin,
    jobTitle: identity.occupation,
    ...(email ? { email: `mailto:${email}` } : {}),
    ...(sameAs.length > 0 ? { sameAs } : {}),
  };

  const website = {
    "@context": "https://schema.org",
    "@type": "WebSite",
    name: identity.siteTitle,
    url: origin,
  };

  const json = JSON.stringify([person, website]).replaceAll("<", "\\u003c");
  return `<script type="application/ld+json">${json}</script>`;
}
