import { describe, expect, it } from "vitest";
import { prefersRawContent } from "./negotiation.ts";

const BROWSER =
  "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";

describe("prefersRawContent", () => {
  it.each([
    ["curl/8.5.0", null],
    ["Wget/1.21", "*/*"],
    ["HTTPie/3.2", "*/*"],
    ["curlie/1.7", BROWSER],
  ])("treats %s as a command-line client", (ua, accept) => {
    expect(prefersRawContent(ua, accept)).toBe(true);
  });

  it("serves HTML to browsers", () => {
    expect(prefersRawContent("Mozilla/5.0", BROWSER)).toBe(false);
  });

  it("serves raw for a bare wildcard without text/html", () => {
    expect(prefersRawContent("python-requests/2.31", "*/*")).toBe(true);
  });

  it("serves raw when the wildcard is listed ahead of text/html", () => {
    expect(prefersRawContent(null, "*/*, text/html")).toBe(true);
  });

  it("assumes a browser when there is no Accept header", () => {
    expect(prefersRawContent("Mozilla/5.0", null)).toBe(false);
  });

  it("serves HTML for other explicit types", () => {
    expect(prefersRawContent("Mozilla/5.0", "application/json")).toBe(false);
  });
});
