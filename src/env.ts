import { defineEnvVars } from "@sveltejs/kit/env";

export const variables = defineEnvVars({
  PUBLIC_POSTHOG_KEY: { public: true, schema: (input) => input },
  PUBLIC_POSTHOG_HOST: { public: true, schema: (input) => input },
});
