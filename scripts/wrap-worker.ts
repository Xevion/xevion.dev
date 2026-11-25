// adapter-cloudflare emits a fetch-only Worker. This adds a `scheduled` handler that forwards each
// cron tick to /internal/cron through the same fetch handler, authenticated by CRON_SECRET, and
// points the generated deploy config at the wrapper.
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const workerDir = fileURLToPath(
  new URL("../.svelte-kit/cloudflare/worker", import.meta.url),
);
const configPath = join(workerDir, "wrangler.json");

const wrapper = `import app from "./index.js";

export default {
  fetch: app.fetch,
  async scheduled(controller, env, ctx) {
    const url = new URL("/internal/cron", env.SITE_ORIGIN);
    url.searchParams.set("cron", controller.cron);
    const request = new Request(url, {
      method: "POST",
      headers: { "x-cron-secret": env.CRON_SECRET ?? "" },
    });
    ctx.waitUntil(
      app.fetch(request, env, ctx).then(async (response) => {
        if (!response.ok) {
          console.error("cron tick failed", controller.cron, response.status, await response.text());
        }
      }),
    );
  },
};
`;

writeFileSync(join(workerDir, "entry.js"), wrapper);

const config = JSON.parse(readFileSync(configPath, "utf8"));
config.main = "entry.js";
writeFileSync(configPath, JSON.stringify(config, null, 2));
console.log(`wrapped worker: ${configPath} -> entry.js`);
