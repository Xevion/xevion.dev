import { describe, expect, it } from "vitest";
import { isIconSegment, parseIconIdentifier } from "./icons.ts";

describe("parseIconIdentifier", () => {
  it("splits a valid identifier", () => {
    expect(parseIconIdentifier("simple-icons:react")).toEqual({
      collection: "simple-icons",
      name: "react",
    });
  });

  it.each(["lucide", ":home", "lucide:", "a:b:c", "Lucide:home", "../x:y"])(
    "rejects %j",
    (value) => {
      expect(parseIconIdentifier(value)).toBeNull();
    },
  );
});

describe("isIconSegment", () => {
  it("rejects path and query characters", () => {
    expect(isIconSegment("home-2")).toBe(true);
    expect(isIconSegment("home.svg")).toBe(false);
    expect(isIconSegment("a/b")).toBe(false);
    expect(isIconSegment("")).toBe(false);
  });
});
