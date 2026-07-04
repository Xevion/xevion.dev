import type {
  ApiProjectDetail,
  ApiSiteIdentity,
  ApiSocialLink,
} from "$lib/bindings";

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

/**
 * Build a project detail page's `SoftwareSourceCode` + `BreadcrumbList`
 * JSON-LD `<script>` element.
 *
 * `codeRepository` is derived the same way as the visible repo link
 * (`ProjectMetaRail.svelte`): `githubRepo` is stored as `owner/repo`, not a
 * full URL. `programmingLanguage` is every tag name, per the decided scope
 * (tags mix languages and domains on this site; not filtered here).
 */
export function projectJsonLdScript(
  project: ApiProjectDetail,
  origin: string,
): string {
  const softwareSourceCode = {
    "@context": "https://schema.org",
    "@type": "SoftwareSourceCode",
    name: project.name,
    description: project.shortDescription,
    ...(project.githubRepo
      ? { codeRepository: `https://github.com/${project.githubRepo}` }
      : {}),
    ...(project.tags.length > 0
      ? { programmingLanguage: project.tags.map((tag) => tag.name) }
      : {}),
  };

  const breadcrumb = {
    "@context": "https://schema.org",
    "@type": "BreadcrumbList",
    itemListElement: [
      { "@type": "ListItem", position: 1, name: "Home", item: origin },
      {
        "@type": "ListItem",
        position: 2,
        name: project.name,
        item: `${origin}/projects/${project.slug}`,
      },
    ],
  };

  const json = JSON.stringify([softwareSourceCode, breadcrumb]).replaceAll(
    "<",
    "\\u003c",
  );
  return `<script type="application/ld+json">${json}</script>`;
}
