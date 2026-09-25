import { readFileSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, it } from "vitest";

import { composite, contrastRatio } from "./color";
import { blockAfter, declarations, themeTokens, tokenColor, type Tokens } from "./tokens";

const CSS = readFileSync(join(process.cwd(), "src/app/globals.css"), "utf8");
const THEMES = [":root", ".dark"] as const;
const BACKDROP = ["--backdrop", "--backdrop-glow-1", "--backdrop-glow-2", "--backdrop-glow-3"];
const SURFACES = ["--glass-panel", "--glass-overlay"];
const BODY_TEXT = ["--foreground", "--muted-foreground"];
const STATUS_TEXT = ["--status-up", "--status-degraded", "--status-down", "--status-unreadable", "--status-unknown"];
const BODY_CONTRAST = 4.5;
const STATUS_CONTRAST = 3;
const FALLBACKS = [
  "@supports not ((backdrop-filter: blur(1px)) or (-webkit-backdrop-filter: blur(1px)))",
  "@media (prefers-reduced-transparency: reduce)",
  "@media (prefers-contrast: more)",
];

function shortfalls(tokens: Tokens, theme: string): string[] {
  const base = tokenColor(tokens, "--backdrop");
  const found: string[] = [];
  for (const point of BACKDROP) {
    const backdrop = composite(tokenColor(tokens, point), base);
    for (const surface of SURFACES) {
      const fill = composite(tokenColor(tokens, surface), backdrop);
      const texts = surface === "--glass-panel" ? [...BODY_TEXT, ...STATUS_TEXT] : BODY_TEXT;
      for (const text of texts) {
        const minimum = STATUS_TEXT.includes(text) ? STATUS_CONTRAST : BODY_CONTRAST;
        const ratio = contrastRatio(composite(tokenColor(tokens, text), fill), fill);
        if (ratio < minimum) {
          found.push(`${theme} ${text} on ${surface} over ${point} is ${ratio.toFixed(2)}, below ${minimum}`);
        }
      }
    }
  }
  return found;
}

describe("glass look", () => {
  it("glass text keeps AA contrast over every backdrop colour in both themes", () => {
    expect(THEMES.flatMap((theme) => shortfalls(themeTokens(CSS, theme), theme))).toEqual([]);
  });

  it("glass falls back to opaque surfaces without backdrop-filter or with reduced transparency or more contrast", () => {
    for (const header of FALLBACKS) {
      const fallback = declarations(blockAfter(CSS, header));
      expect(fallback.get("--glass-blur-panel")).toBe("0px");
      expect(fallback.get("--glass-blur-overlay")).toBe("0px");
      for (const theme of THEMES) {
        const tokens = new Map([...themeTokens(CSS, theme), ...fallback]);
        for (const surface of SURFACES) {
          expect(tokenColor(tokens, surface).alpha, `${header} ${theme} ${surface}`).toBe(1);
        }
      }
    }
  });

  it("glass surfaces blur through the utilities alone", () => {
    for (const utility of ["glass-panel", "glass-overlay"]) {
      const block = blockAfter(CSS, `@utility ${utility} {`);
      expect(block).toContain("-webkit-backdrop-filter:");
      expect(block).toMatch(/\n\s+backdrop-filter:/);
    }
  });
});
