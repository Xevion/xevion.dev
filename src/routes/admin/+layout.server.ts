import type { LayoutServerLoad } from "./$types";

export const load: LayoutServerLoad = async ({ locals }) => {
  if (!locals.user) return {};

  return {
    session: {
      authenticated: true,
      username: locals.user.username,
    },
  };
};
