import { drizzle } from "drizzle-orm/d1";
import * as schema from "./schema.ts";

export const getDb = (d1: D1Database) => drizzle(d1, { schema });
export type Db = ReturnType<typeof getDb>;

// D1 reports constraint failures only as message text.
export function isUniqueViolation(err: unknown): boolean {
  return (
    err instanceof Error && /UNIQUE constraint failed/i.test(errorText(err))
  );
}

function errorText(err: Error): string {
  return err.cause instanceof Error
    ? `${err.message} ${err.cause.message}`
    : err.message;
}
