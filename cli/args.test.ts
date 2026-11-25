import { describe, expect, it } from "vitest";
import { parseCreateTags, parseHexColor, parseUpdateTags } from "./args.ts";

describe("parseHexColor", () => {
  it("accepts bare and prefixed values and normalizes to lowercase", () => {
    expect(parseHexColor("3B82F6")).toBe("3b82f6");
    expect(parseHexColor("#3b82f6")).toBe("3b82f6");
  });

  it("passes the empty clear sentinel through", () => {
    expect(parseHexColor("")).toBe("");
  });

  it.each(["xyz", "abc", "abcdef0", "#12345g"])("rejects %j", (bad) => {
    expect(() => parseHexColor(bad)).toThrow("is not a 6-digit hex color");
  });
});

describe("tag parsing", () => {
  it("create tags accept an optional plus prefix", () => {
    expect(parseCreateTags("rust, +web ,,cli")).toEqual(["rust", "web", "cli"]);
  });

  it("update tags require a prefix", () => {
    expect(parseUpdateTags("+new,-old")).toEqual([
      { op: "add", tag: "new" },
      { op: "remove", tag: "old" },
    ]);
    expect(() => parseUpdateTags("bare")).toThrow(
      "Tag 'bare' requires prefix: use +bare to add, -bare to remove",
    );
    expect(() => parseUpdateTags("+")).toThrow(
      "Tag name cannot be empty after '+'",
    );
  });
});
