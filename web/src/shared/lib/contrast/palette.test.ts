import { readFileSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, it } from "vitest";

import { composite, contrastRatio } from "./color";
import { themeTokens, tokenColor, type Tokens } from "./tokens";

const CSS = readFileSync(join(process.cwd(), "src/app/globals.css"), "utf8");
const THEMES = [":root", ".dark"] as const;
const BACKDROP = ["--backdrop", "--backdrop-glow-1", "--backdrop-glow-2", "--backdrop-glow-3"];
export const PALETTE = ["neutral", "blue", "cyan", "teal", "green", "lime", "amber", "orange", "red", "pink", "violet", "indigo"];
const TEXT = 3;
const FILL = 3;

function shortfalls(tokens: Tokens, theme: string, names: string[]): string[] {
  const found: string[] = [];
  const base = tokenColor(tokens, "--backdrop");
  const label = tokenColor(tokens, "--primary-foreground");
  for (const name of names) {
    const colour = tokenColor(tokens, `--palette-${name}`);
    for (const point of BACKDROP) {
      const panel = composite(tokenColor(tokens, "--glass-panel"), composite(tokenColor(tokens, point), base));
      const asText = contrastRatio(colour, panel);
      if (asText < TEXT) {
        found.push(`${theme} ${name} as text over ${point} is ${asText.toFixed(2)}, below ${TEXT}`);
      }
    }
    const asFill = contrastRatio(label, colour);
    if (asFill < FILL) {
      found.push(`${theme} ${name} as a fill under its label is ${asFill.toFixed(2)}, below ${FILL}`);
    }
  }
  return found;
}

describe("the palette", () => {
  it("has every colour in both themes", () => {
    for (const theme of THEMES) {
      const tokens = themeTokens(CSS, theme);
      expect(PALETTE.filter((name) => !tokens.has(`--palette-${name}`)), theme).toEqual([]);
    }
  });

  it("reads as text on a panel and as a fill under its label, in both themes", () => {
    expect(THEMES.flatMap((theme) => shortfalls(themeTokens(CSS, theme), theme, PALETTE))).toEqual([]);
  });

  it("catches a colour too light to read", () => {
    const pale = CSS.replace("--palette-orange: oklch(0.6 0.16 45);", "--palette-orange: oklch(0.85 0.08 45);");
    expect(pale).not.toBe(CSS);
    expect(shortfalls(themeTokens(pale, ":root"), ":root", ["orange"]).length).toBeGreaterThan(0);
  });
});
