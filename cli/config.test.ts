import {
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { Config } from "./config.ts";

let dir: string;
beforeEach(() => {
  dir = mkdtempSync(join(tmpdir(), "xevion-config-"));
});
afterEach(() => {
  rmSync(dir, { recursive: true, force: true });
});

function two(): Config {
  const config = new Config(join(dir, "config.toml"));
  config.setTarget("local", "http://localhost:10237", "t1");
  config.setTarget("production", "https://xevion.dev", "t2");
  return config;
}

describe("Config", () => {
  it("adopts the first target as default only", () => {
    expect(two().default).toBe("local");
  });

  it("resolves the override, then the default", () => {
    const config = two();
    expect(config.resolve().name).toBe("local");
    const { name, entry } = config.resolve("production");
    expect(name).toBe("production");
    expect(entry.token).toBe("t2");
  });

  it("errors without a token, for unknown names and with no targets", () => {
    const tokenless = new Config(join(dir, "c.toml"));
    tokenless.setTarget("local", "http://localhost:10237");
    expect(() => tokenless.resolve()).toThrow("is not authorized");
    expect(() => two().resolve("nope")).toThrow("no API target named 'nope'");
    expect(() => new Config(join(dir, "c.toml")).resolve()).toThrow(
      "no API target selected",
    );
  });

  it("keeps the entry when its token is cleared", () => {
    const config = two();
    config.clearToken("local");
    expect(config.urlFor("local")).toBe("http://localhost:10237");
    expect(() => config.resolve("local")).toThrow();
  });

  it("requires an existing target to become the default", () => {
    const config = two();
    config.setDefault("production");
    expect(config.default).toBe("production");
    expect(() => config.setDefault("ghost")).toThrow("no API target named");
  });

  it("clears a dangling default on removal", () => {
    const config = two();
    config.removeTarget("local");
    expect(config.default).toBeUndefined();
    expect(config.hasTarget("local")).toBe(false);
    expect(() => config.removeTarget("local")).toThrow();
  });

  it("renames preserving the default and rejects collisions", () => {
    const config = two();
    config.renameTarget("local", "dev");
    expect(config.hasTarget("local")).toBe(false);
    expect(config.default).toBe("dev");
    expect(() => config.renameTarget("dev", "production")).toThrow(
      "already exists",
    );
  });

  it("round-trips through a 0600 file in the existing format", () => {
    const config = two();
    config.save();
    expect(statSync(config.path).mode & 0o777).toBe(0o600);
    expect(readFileSync(config.path, "utf8")).toBe(
      [
        'default = "local"',
        "",
        "[api.local]",
        'url = "http://localhost:10237"',
        'token = "t1"',
        "",
        "[api.production]",
        'url = "https://xevion.dev"',
        'token = "t2"',
        "",
      ].join("\n"),
    );
    const loaded = Config.load(config.path);
    expect(loaded.default).toBe("local");
    expect(loaded.resolve("production").entry.token).toBe("t2");
  });

  it("loads a hand-written config and tolerates a missing file", () => {
    const path = join(dir, "hand.toml");
    writeFileSync(
      path,
      'default = "production"\n\n[api.production]\nurl = "https://xevion.dev"\ntoken = "xev_abc"\n',
    );
    expect(Config.load(path).resolve().entry.token).toBe("xev_abc");
    expect(Config.load(join(dir, "missing.toml")).names()).toEqual([]);
  });

  it("rejects invalid TOML", () => {
    const path = join(dir, "bad.toml");
    writeFileSync(path, "default = ");
    expect(() => Config.load(path)).toThrow("is not valid TOML");
  });
});
