export interface SitemapUrl {
  slug: string;
  lastmod: Date;
}

export function renderRobots(origin: string): string {
  return [
    "User-agent: *",
    "Allow: /",
    "Disallow: /admin/",
    "Disallow: /api/",
    "Disallow: /internal/",
    "",
    `Sitemap: ${origin}/sitemap.xml`,
    "",
  ].join("\n");
}

function urlEntry(loc: string, lastmod?: Date): string {
  const mod = lastmod
    ? `    <lastmod>${lastmod.toISOString().slice(0, 10)}</lastmod>\n`
    : "";
  return `  <url>\n    <loc>${loc}</loc>\n${mod}  </url>\n`;
}

/** Homepage, PGP page and every public project, with `lastmod` on the project pages. */
export function renderSitemap(origin: string, projects: SitemapUrl[]): string {
  return (
    '<?xml version="1.0" encoding="UTF-8"?>\n' +
    '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n' +
    urlEntry(`${origin}/`) +
    urlEntry(`${origin}/pgp`) +
    projects
      .map((p) =>
        urlEntry(`${origin}/projects/${encodeURIComponent(p.slug)}`, p.lastmod),
      )
      .join("") +
    "</urlset>\n"
  );
}
