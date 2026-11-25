import {
  chmodSync,
  existsSync,
  mkdirSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { parse, stringify } from "smol-toml";
import { CliError } from "./errors.ts";

/** A single named API target. */
export interface ApiEntry {
  url: string;
  token?: string;
}

/** Platform config directory, matching where the previous CLI kept its config. */
function platformConfigDir(): string | undefined {
  const home = homedir();
  if (process.platform === "win32") {
    const appData = process.env.APPDATA;
    return appData ? join(appData, "xevion", "xevion", "config") : undefined;
  }
  if (process.platform === "darwin") {
    return home
      ? join(home, "Library", "Application Support", "dev.xevion.xevion")
      : undefined;
  }
  const xdg = process.env.XDG_CONFIG_HOME;
  if (xdg && xdg.startsWith("/")) return join(xdg, "xevion");
  return home ? join(home, ".config", "xevion") : undefined;
}

/**
 * `--config`/`XEVION_CONFIG` override, else the platform config directory.
 * Errors rather than silently falling back to the working directory.
 */
export function resolveConfigPath(override?: string): string {
  if (override) return override;
  const fromEnv = process.env.XEVION_CONFIG;
  if (fromEnv) return fromEnv;
  const dir = platformConfigDir();
  if (!dir) {
    throw CliError.config(
      "could not determine a config directory",
      "set XEVION_CONFIG=/path/to/config.toml or pass --config <file>",
    );
  }
  return join(dir, "config.toml");
}

/**
 * TOML-backed set of named API targets with a default selection:
 *
 *   default = "local"
 *
 *   [api.local]
 *   url = "http://localhost:10237"
 *   token = "xev_..."
 */
export class Config {
  default?: string;
  readonly api = new Map<string, ApiEntry>();

  constructor(readonly path: string) {}

  /** Load from `path`; a missing file yields an empty config. */
  static load(path: string): Config {
    const config = new Config(path);
    if (!existsSync(path)) return config;

    let raw: string;
    try {
      raw = readFileSync(path, "utf8");
    } catch (e) {
      throw CliError.config(
        `could not access config at ${path}: ${errorMessage(e)}`,
      );
    }

    let doc: Record<string, unknown>;
    try {
      doc = parse(raw);
    } catch (e) {
      throw CliError.config(
        `config at ${path} is not valid TOML: ${errorMessage(e)}`,
      );
    }

    if (doc.default !== undefined) {
      if (typeof doc.default !== "string") {
        throw CliError.config(
          `config at ${path} is not valid: "default" must be a string`,
        );
      }
      config.default = doc.default;
    }
    const api = doc.api;
    if (api !== undefined) {
      if (typeof api !== "object" || api === null || Array.isArray(api)) {
        throw CliError.config(
          `config at ${path} is not valid: "api" must be a table`,
        );
      }
      for (const [name, entry] of Object.entries(api)) {
        const e = entry as Record<string, unknown> | null;
        if (typeof e?.url !== "string") {
          throw CliError.config(
            `config at ${path} is not valid: [api.${name}] needs a string "url"`,
          );
        }
        if (e.token !== undefined && typeof e.token !== "string") {
          throw CliError.config(
            `config at ${path} is not valid: [api.${name}] "token" must be a string`,
          );
        }
        config.api.set(name, { url: e.url, token: e.token });
      }
    }
    return config;
  }

  /** Atomically write the config (temp file + rename), locked to the owner. */
  save(): void {
    const apis: Record<string, Record<string, string>> = {};
    for (const name of [...this.api.keys()].sort()) {
      const entry = this.api.get(name)!;
      apis[name] = {
        url: entry.url,
        ...(entry.token === undefined ? {} : { token: entry.token }),
      };
    }
    const doc: Record<string, unknown> = {};
    if (this.default !== undefined) doc.default = this.default;
    if (Object.keys(apis).length > 0) doc.api = apis;
    const content = stringify(doc);

    const tmp = `${this.path}.${process.pid}.tmp`;
    try {
      mkdirSync(dirname(this.path), { recursive: true });
      writeFileSync(tmp, content, { mode: 0o600 });
      chmodSync(tmp, 0o600);
      renameSync(tmp, this.path);
    } catch (e) {
      rmSync(tmp, { force: true });
      throw CliError.config(
        `could not access config at ${this.path}: ${errorMessage(e)}`,
      );
    }
  }

  private targetName(override?: string): string | undefined {
    return override ?? this.default;
  }

  /**
   * The selected target with its token, for issuing requests. The help text
   * adapts to what is actually configured.
   */
  resolve(override?: string): {
    name: string;
    entry: ApiEntry & { token: string };
  } {
    const name = this.targetName(override);
    if (name === undefined) {
      throw CliError.config("no API target selected", this.noTargetHelp());
    }
    const entry = this.api.get(name);
    if (!entry) throw this.unknownTarget(name);
    if (entry.token === undefined) {
      throw CliError.config(
        `API target '${name}' is not authorized`,
        `authorize it: xevion login --api ${name}`,
      );
    }
    return { name, entry: { ...entry, token: entry.token } };
  }

  private noTargetHelp(): string {
    const existing = this.names()[0];
    return existing === undefined
      ? "authorize a target first: xevion login --api <name> --url <url>"
      : `select a target with --api <name> (e.g. --api ${existing}) or set a default: xevion targets use ${existing}`;
  }

  private unknownTarget(name: string): CliError {
    const names = this.names();
    const help =
      names.length === 0
        ? "no targets are configured yet: xevion login --api <name> --url <url>"
        : `configured targets: ${names.join(", ")}`;
    return CliError.config(`no API target named '${name}'`, help);
  }

  urlFor(name: string): string | undefined {
    return this.api.get(name)?.url;
  }

  hasTarget(name: string): boolean {
    return this.api.has(name);
  }

  isAuthorized(name: string): boolean {
    return this.api.get(name)?.token !== undefined;
  }

  names(): string[] {
    return [...this.api.keys()].sort();
  }

  /** Upsert a target, making it the default if none is set. */
  setTarget(name: string, url: string, token?: string): void {
    this.api.set(name, { url, token });
    this.default ??= name;
  }

  setDefault(name: string): void {
    if (!this.hasTarget(name)) throw this.unknownTarget(name);
    this.default = name;
  }

  /** Remove a target, clearing the default if it pointed here. */
  removeTarget(name: string): ApiEntry {
    const entry = this.api.get(name);
    if (!entry) throw this.unknownTarget(name);
    this.api.delete(name);
    if (this.default === name) this.default = undefined;
    return entry;
  }

  /** Rename a target, preserving its URL, token and default status. */
  renameTarget(from: string, to: string): void {
    const entry = this.api.get(from);
    if (!entry) throw this.unknownTarget(from);
    if (this.hasTarget(to)) {
      throw CliError.config(`a target named '${to}' already exists`);
    }
    this.api.delete(from);
    this.api.set(to, entry);
    if (this.default === from) this.default = to;
  }

  /** Clear a target's token (logout) without removing the entry. */
  clearToken(name: string): void {
    const entry = this.api.get(name);
    if (entry) entry.token = undefined;
  }
}

function errorMessage(e: unknown): string {
  return e instanceof Error ? e.message : String(e);
}
