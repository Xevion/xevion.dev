import type { LayoutServerLoad } from "./$types";
import { getOGImageUrl } from "$lib/og-types";

export const load: LayoutServerLoad = async ({ url }) => {
  return {
    metadata: {
      title: "Xevion.dev",
      description:
        "The personal website of Xevion, a full-stack software developer.",
      ogImage: getOGImageUrl({ type: "index" }),
      url: url.toString(),
    },
  };
};
