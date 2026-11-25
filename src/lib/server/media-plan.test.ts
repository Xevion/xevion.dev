import { describe, expect, it } from "vitest";
import {
  classifyMime,
  extensionForMime,
  isTooSmall,
  isValidBlurhash,
  planVariants,
} from "./media-plan.ts";

describe("planVariants", () => {
  it("downscales a wide image and preserves aspect ratio", () => {
    expect(planVariants(3200, 1800)).toEqual([
      { name: "thumb", width: 300, height: 169 },
      { name: "medium", width: 800, height: 450 },
      { name: "full", width: 1600, height: 900 },
    ]);
  });

  it("never enlarges a source narrower than a variant", () => {
    expect(planVariants(1000, 500)).toEqual([
      { name: "thumb", width: 300, height: 150 },
      { name: "medium", width: 800, height: 400 },
      { name: "full", width: 1000, height: 500 },
    ]);
  });
});

describe("isTooSmall", () => {
  it.each([
    [300, 300, false],
    [299, 2000, true],
    [2000, 299, true],
  ])("%ix%i -> %s", (w, h, expected) => {
    expect(isTooSmall(w, h)).toBe(expected);
  });
});

describe("classifyMime", () => {
  it.each([
    ["image/avif", "image"],
    ["video/quicktime", "video"],
    ["image/svg+xml", null],
    ["application/pdf", null],
  ])("%s -> %s", (mime, expected) => {
    expect(classifyMime(mime)).toBe(expected);
  });

  it("maps video mimes to their container extensions", () => {
    expect(extensionForMime("video/quicktime")).toBe("mov");
    expect(extensionForMime("image/jpeg")).toBe("jpg");
  });
});

describe("isValidBlurhash", () => {
  it("accepts a real 4x3 hash and rejects junk", () => {
    expect(isValidBlurhash("LEHV6nWB2yk8pyo0adR*.7kCMdnj")).toBe(true);
    expect(isValidBlurhash("")).toBe(false);
    expect(isValidBlurhash("short")).toBe(false);
    expect(isValidBlurhash("has spaces in it here")).toBe(false);
    expect(isValidBlurhash("a".repeat(200))).toBe(false);
  });
});
