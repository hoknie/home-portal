import { describe, expect, it } from "vitest";

import { type Color, composite, contrastRatio, parseOklch } from "./color";

const WHITE: Color = { red: 1, green: 1, blue: 1, alpha: 1 };
const BLACK: Color = { red: 0, green: 0, blue: 0, alpha: 1 };

describe("colour maths", () => {
  it("reads oklch white and black as sRGB white and black", () => {
    const white = parseOklch("oklch(1 0 0)");
    const black = parseOklch("oklch(0 0 0)");
    expect(white.red).toBeCloseTo(1, 3);
    expect(white.green).toBeCloseTo(1, 3);
    expect(white.blue).toBeCloseTo(1, 3);
    expect(black.red).toBeCloseTo(0, 3);
  });

  it("reads alpha as a fraction or a percentage", () => {
    expect(parseOklch("oklch(1 0 0 / 55%)").alpha).toBeCloseTo(0.55);
    expect(parseOklch("oklch(0.5 0.1 262 / 0.3)").alpha).toBeCloseTo(0.3);
  });

  it("reads a known saturated colour as its sRGB value", () => {
    const red = parseOklch("oklch(0.628 0.2577 29.23)");
    expect(red.red).toBeCloseTo(1, 2);
    expect(red.green).toBeCloseTo(0, 2);
    expect(red.blue).toBeCloseTo(0, 2);
  });

  it("refuses text that is not an oklch colour", () => {
    expect(() => parseOklch("var(--card)")).toThrow("not an oklch colour");
  });

  it("measures white on black as 21 to 1", () => {
    expect(contrastRatio(WHITE, BLACK)).toBeCloseTo(21);
    expect(contrastRatio(BLACK, WHITE)).toBeCloseTo(21);
  });

  it("composites a translucent colour over an opaque one", () => {
    const half = composite({ ...WHITE, alpha: 0.5 }, BLACK);
    expect(half.red).toBeCloseTo(0.5);
    expect(half.alpha).toBe(1);
  });
});
