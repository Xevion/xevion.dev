import type { Db } from "#lib/server/db/index.ts";
import type { AuthUser } from "#lib/server/sessions.ts";
import type { ApiErrorCode } from "#lib/server/api.ts";

declare global {
  namespace App {
    interface Error {
      code?: ApiErrorCode;
      fieldErrors?: Record<string, string>;
      errorId?: string;
    }
    interface Locals {
      db: Db;
      user: AuthUser | null;
    }
    interface PageState {
      discordModal?: {
        open: boolean;
        username: string;
      };
    }
  }
}

export {};
