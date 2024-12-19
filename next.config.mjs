// @ts-check

import withPlaiceholder from "@plaiceholder/next";

/**
 * Run `build` or `dev` with `SKIP_ENV_VALIDATION` to skip env validation.
 * This is especially useful for Docker builds.
 */
if (!process.env.SKIP_ENV_VALIDATION) await import("./src/env/server.mjs");

/**
 *
 * @param {string} text The string to search around with the pattern in.
 * @param {string} pattern The strict, text only pattern to search for.
 * @param {number} nth The index of the pattern to find.
 * @returns
 */
function nthIndex(text, pattern, nth) {
  const L = text.length;
  let i = -1;
  while (nth-- && i++ < L) {
    i = text.indexOf(pattern, i);
    if (i < 0) break;
  }
  return i;
}

const v2_redirects = [
  "/2020/12/04/jekyll-github-pages-and-azabani",
  "/2021/02/25/project-facelift-new-and-old",
  "/2022/03/29/runnerspace-built-in-under-30-hours",
  "/2022/07/16/restricted-memory-and-data-framing-tricks",
  "/drafts/2022-09-19-presenting-to-humans/",
  "/photography",
].map((url) => {
  // If the URL starts with /2, redirect to the new blog. Otherwise, redirect to the old v2 blog to maintain URLs.
  if (url.startsWith("/2"))
    return {
      source: url,
      destination: `https://undefined.behavio.rs/posts${url.slice(
        nthIndex(url, "/", 4),
      )}`,
      permanent: false,
    };

  return {
    source: url,
    destination: `https://v2.xevion.dev${url}`,
    permanent: false,
  };
});

/** @type {import("next").NextConfig} */
const config = {
  reactStrictMode: true,
  async redirects() {
    // Source cannot end with / slash
    return [
      { source: "/resume", destination: "/resume.pdf", permanent: false },
      ...v2_redirects,
    ];
  },
};
export default withPlaiceholder(config);
