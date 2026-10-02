import { readFileSync } from "node:fs";
import { join } from "node:path";

import { expect, it } from "vitest";

import { blockAfter, declarations } from "@/shared/lib/contrast";

const CSS = readFileSync(join(process.cwd(), "src/app/globals.css"), "utf8");
const REDUCED = "@media (prefers-reduced-motion: reduce)";

function milliseconds(value: string): number {
  return Number(value.replace("ms", ""));
}

it("every motion duration lies between 150 and 250 ms", () => {
  const tokens = declarations(blockAfter(CSS, ":root {"));
  for (const name of ["--motion-fast", "--motion-base", "--motion-slow"]) {
    const value = milliseconds(tokens.get(name) ?? "");
    expect(value, name).toBeGreaterThanOrEqual(150);
    expect(value, name).toBeLessThanOrEqual(250);
  }
});

it("reduced motion stops view transitions, enter and exit animations, and transitions", () => {
  const reduced = CSS.slice(CSS.indexOf(REDUCED));
  expect(reduced).toContain(REDUCED);
  for (const selector of ["::view-transition-old(*)", "::view-transition-new(*)", "::view-transition-group(*)", ".animate-in", ".animate-out"]) {
    expect(reduced, selector).toContain(selector);
  }
  expect(reduced).toMatch(/animation: none !important/);
  expect(reduced).toMatch(/transition-duration: 0s !important/);
});

it("the surfaces of the scale exist, and only the raised one blurs", () => {
  for (const surface of ["surface-panel", "surface-raised", "surface-solid", "surface-inset", "surface-tint"]) {
    const block = blockAfter(CSS, `@utility ${surface} {`);
    expect(block, surface).toContain("background-color");
    expect(/backdrop-filter/.test(block), surface).toBe(surface === "surface-raised");
  }
});

it("the new page appears only after the old one has gone, so the two never overlap", () => {
  const leaving = blockAfter(CSS, "::view-transition-old(.page-fade) {");
  const arriving = blockAfter(CSS, "::view-transition-new(.page-fade) {");
  expect(leaving).toContain("var(--motion-fast)");
  expect(arriving).toMatch(/var\(--motion-base\) var\(--motion-ease\) var\(--motion-fast\)/);
  expect(blockAfter(CSS, "::view-transition-group(.page-fade) {")).toContain("animation: none");
});

it("no transition layer is lifted above the page, so dialogs and menus keep their order", () => {
  const transitions = CSS.slice(CSS.indexOf("::view-transition"));
  expect(transitions).not.toMatch(/z-index/);
  expect(blockAfter(CSS, "::view-transition-old(root),")).toContain("animation: none");
});
