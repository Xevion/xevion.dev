// SQLite INSERT emission shared by the legacy import scripts.

export type Value =
  | string
  | number
  | boolean
  | null
  | Date
  | object
  | undefined;

function quote(text: string): string {
  return `'${text.replaceAll("'", "''")}'`;
}

function literal(value: Value): string {
  if (value === null || value === undefined) return "NULL";
  if (typeof value === "boolean") return value ? "1" : "0";
  if (typeof value === "number" || typeof value === "bigint")
    return String(value);
  if (value instanceof Date) return String(Math.floor(value.getTime() / 1000));
  if (typeof value === "object") return quote(JSON.stringify(value));
  return quote(value);
}

// D1 rejects a single statement over 100 KB.
const STATEMENT_LIMIT = 100_000;

export class SqlScript {
  readonly statements: string[] = [];

  insert(table: string, row: Record<string, Value>, verb = "INSERT"): void {
    const columns = Object.keys(row);
    const sql = `${verb} INTO \`${table}\` (${columns.map((c) => `\`${c}\``).join(", ")}) VALUES (${columns
      .map((c) => literal(row[c]))
      .join(", ")});`;
    if (sql.length > STATEMENT_LIMIT) {
      throw new Error(
        `${table} row exceeds D1's statement limit (${sql.length} bytes)`,
      );
    }
    this.statements.push(sql);
  }

  /** The deferred check keeps insert order flexible. */
  toString(): string {
    return ["PRAGMA defer_foreign_keys = true;", ...this.statements, ""].join(
      "\n",
    );
  }
}
