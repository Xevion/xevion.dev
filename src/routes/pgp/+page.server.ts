import { redirect } from "@sveltejs/kit";
import { PGP_KEY_METADATA } from "#lib/pgp/key-info.ts";
import { prefersRawContent } from "#lib/server/negotiation.ts";
import { publicKey } from "#lib/server/static-text.ts";
import type { PageServerLoad } from "./$types";

export const load: PageServerLoad = ({ request }) => {
  // curl and friends that explicitly ask for text/html still get the key, not the page.
  if (
    prefersRawContent(
      request.headers.get("user-agent"),
      request.headers.get("accept"),
    )
  ) {
    redirect(303, "/publickey.asc");
  }
  return { key: { ...PGP_KEY_METADATA, content: publicKey } };
};
