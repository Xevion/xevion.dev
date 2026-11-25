import type { Cookies } from "@sveltejs/kit";
import { dev } from "$app/env";

export const SESSION_COOKIE = "session";

export function setSessionCookie(
  cookies: Cookies,
  token: string,
  expiresAt: Date,
): void {
  cookies.set(SESSION_COOKIE, token, {
    path: "/",
    httpOnly: true,
    secure: !dev,
    sameSite: "lax",
    expires: expiresAt,
  });
}

export function clearSessionCookie(cookies: Cookies): void {
  cookies.delete(SESSION_COOKIE, { path: "/" });
}
