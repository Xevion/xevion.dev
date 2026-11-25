import { describe, expect, it } from "vitest";
import { isHexColor, isUuid, normalizeHexColor, slugify } from "./slug.ts";

describe("slugify", () => {
  it.each([
    ["Hello World", "hello-world"],
    ["  Spaced   Out  ", "spaced-out"],
    ["snake_case.and-kebab", "snake-case-and-kebab"],
    ["C++ & Rust!", "c-rust"],
    ["Café Münster 2", "café-münster-2"],
    ["---", ""],
    ["", ""],
  ])("%j -> %j", (input, expected) => {
    expect(slugify(input)).toBe(expected);
  });
});

describe("colors", () => {
  it("accepts exactly six hex digits without a hash", () => {
    expect(isHexColor("3b82f6")).toBe(true);
    expect(isHexColor("3B82F6")).toBe(true);
    expect(isHexColor("#3b82f6")).toBe(false);
    expect(isHexColor("3b82f")).toBe(false);
    expect(isHexColor("3b82f6ff")).toBe(false);
    expect(isHexColor("zzzzzz")).toBe(false);
  });

  it("normalizes to bare lowercase", () => {
    expect(normalizeHexColor("#6366F1")).toBe("6366f1");
    expect(normalizeHexColor("6366F1")).toBe("6366f1");
    expect(normalizeHexColor("##6366F1")).toBeNull();
    expect(normalizeHexColor("blue")).toBeNull();
  });
});

describe("isUuid", () => {
  it("distinguishes ids from slugs", () => {
    expect(isUuid(crypto.randomUUID())).toBe(true);
    expect(isUuid("my-project")).toBe(false);
    expect(isUuid("12345678-1234-1234-1234-12345678901")).toBe(false);
  });
});
