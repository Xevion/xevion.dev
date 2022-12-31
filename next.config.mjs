// @ts-check
/**
 * Run `build` or `dev` with `SKIP_ENV_VALIDATION` to skip env validation.
 * This is especially useful for Docker builds.
 */
!process.env.SKIP_ENV_VALIDATION && (await import("./src/env/server.mjs"));


const v2_redirects = [
    '/2020/12/04/jekyll-github-pages-and-azabani',
    '/2021/02/25/project-facelift-new-and-old',
    '/2022/03/29/runnerspace-built-in-under-30-hours',
    '/2022/07/16/restricted-memory-and-data-framing-tricks',
    '/drafts/presenting-to-humans',
    '/photography'
].map(url => {
    return {
        source: url, destination: `https://v2.xevion.dev${url}`, permanent: false
    }
})

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
            ...v2_redirects
        ]
    }
};
export default config;
