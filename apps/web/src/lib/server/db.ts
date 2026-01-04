import { drizzle } from "drizzle-orm/postgres-js";
import postgres from "postgres";
import * as schema from "@xevion/db";
import { projectsRelations } from "@xevion/db";

const extendedSchema = {
  ...schema,
  relations_projects: projectsRelations,
};

let _db: ReturnType<typeof drizzle> | null = null;

export function getDb() {
  if (!_db) {
    const connectionString = process.env.DATABASE_URL;

    if (!connectionString) {
      throw new Error("DATABASE_URL environment variable is not set");
    }

    const sql = postgres(connectionString);
    _db = drizzle(sql, { schema: extendedSchema });
  }

  return _db;
}

// For backward compatibility
export const db = new Proxy({} as ReturnType<typeof drizzle>, {
  get(target, prop) {
    return getDb()[prop as keyof ReturnType<typeof drizzle>];
  },
});
