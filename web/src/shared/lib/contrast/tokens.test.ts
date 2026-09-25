import { readFileSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, it } from "vitest";

import { blockAfter, themeTokens, tokenColor, tokenValue } from "./tokens";

const SAMPLE = [
  ":root {",
  "  --paper: oklch(1 0 0);",
  "  --card: var(--paper);",
  "}",
  "",
  ".dark {",
  "  --paper: oklch(0.2 0 0 / 50%);",
  "}",
  "",
  "@media (prefers-contrast: more) {",
  "  :root { --card: oklch(1 0 0); }",
  "}",
  "",
].join("\n");

describe("design token reader", () => {
  it("reads the top-level theme blocks and follows var references", () => {
    const light = themeTokens(`\n${SAMPLE}`, ":root");
    expect(tokenValue(light, "--card")).toBe("oklch(1 0 0)");
    expect(tokenColor(themeTokens(`\n${SAMPLE}`, ".dark"), "--paper").alpha).toBeCloseTo(0.5);
  });

  it("reads a nested block by its header", () => {
    expect(blockAfter(SAMPLE, "@media (prefers-contrast: more)")).toContain("--card: oklch(1 0 0)");
  });

  it("names the token that is missing", () => {
    expect(() => tokenValue(themeTokens(`\n${SAMPLE}`, ":root"), "--glass")).toThrow("missing token --glass");
    expect(() => blockAfter(SAMPLE, "@supports")).toThrow("no block starts with @supports");
  });

  it("reads both themes of the real stylesheet", () => {
    const css = readFileSync(join(process.cwd(), "src/app/globals.css"), "utf8");
    expect(tokenColor(themeTokens(css, ":root"), "--foreground").alpha).toBe(1);
    expect(tokenColor(themeTokens(css, ".dark"), "--foreground").alpha).toBe(1);
  });
});
