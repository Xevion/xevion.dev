import { redirect } from "@sveltejs/kit";
import type { LayoutServerLoad } from "./$types";

export const load: LayoutServerLoad = async ({ request, url }) => {
  // Login page doesn't require authentication
  if (url.pathname === "/admin/login") {
    return {};
  }

  // Read trusted header from Rust proxy (cannot be spoofed by client)
  const sessionUser = request.headers.get("x-session-user");

  if (!sessionUser) {
    // Not authenticated - redirect to login with next parameter
    throw redirect(
      302,
      `/admin/login?next=${encodeURIComponent(url.pathname + url.search)}`
    );
  }

  return {
    session: {
      authenticated: true,
      username: sessionUser,
    },
  };
};
