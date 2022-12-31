// @ts-check
/**
 * Run `build` or `dev` with `SKIP_ENV_VALIDATION` to skip env validation.
 * This is especially useful for Docker builds.
 */
!process.env.SKIP_ENV_VALIDATION && (await import("./src/env/server.mjs"));

/** @type {import("next").NextConfig} */
const config = {
    reactStrictMode: true,
    swcMinify: true,
    i18n: {
        locales: ["en"],
        defaultLocale: "en",
    },
    async redirects() {
        // Source cannot end with / slash
        return [
            {source: '/resume', destination: '/resume.pdf', permanent: false},
            {source: '/2020/12/04/jekyll-github-pages-and-azabani', destination: 'https://v2.xevion.dev/2020/12/04/jekyll-github-pages-and-azabani/', permanent: false},
            {source: '/2021/02/25/project-facelift-new-and-old', destination: 'https://v2.xevion.dev/2021/02/25/project-facelift-new-and-old/', permanent: false},
            {source: '/2022/03/29/runnerspace-built-in-under-30-hours', destination: 'https://v2.xevion.dev/2022/03/29/runnerspace-built-in-under-30-hours/', permanent: false},
            {source: '/2022/07/16/restricted-memory-and-data-framing-tricks', destination: 'https://v2.xevion.dev/2022/07/16/restricted-memory-and-data-framing-tricks/', permanent: false},
            {source: '/photography', destination: 'https://v2.xevion.dev/photography/', permanent: false},
            {source: '/drafts/presenting-to-humans', destination: 'https://v2.xevion.dev/drafts/presenting-to-humans/', permanent: false}
        ]
    }
};
export default config;
