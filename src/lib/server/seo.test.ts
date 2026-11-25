import { describe, expect, it } from "vitest";
import { renderSitemap } from "./seo.ts";

describe("renderSitemap", () => {
  it("lists the fixed pages and project pages with a date-only lastmod", () => {
    const xml = renderSitemap("https://xevion.dev", [
      { slug: "my-app", lastmod: new Date("2026-02-03T23:59:59Z") },
    ]);
    expect(xml).toContain("<loc>https://xevion.dev/</loc>");
    expect(xml).toContain("<loc>https://xevion.dev/pgp</loc>");
    expect(xml).toContain(
      "<loc>https://xevion.dev/projects/my-app</loc>\n    <lastmod>2026-02-03</lastmod>",
    );
  });
});
