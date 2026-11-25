import type { SessionType } from "./SessionType";

/**
 * A session as shown in the admin sessions list.
 */
export type ApiSession = { id: string, sessionType: SessionType, label?: string, createdAt: string, lastActiveAt: string, expiresAt: string, 
/**
 * True for the session making this request (don't let the user lock themselves out blindly).
 */
current: boolean, };
