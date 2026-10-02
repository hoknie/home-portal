import { readFileSync } from "node:fs";
import { join } from "node:path";

import { expect, it } from "vitest";

import { TONE_NAMES, TONES, toneOf } from "./tones";

const CSS = readFileSync(join(process.cwd(), "src/app/globals.css"), "utf8");

it("every tone draws text, a fill, a badge and the three button styles from a colour the theme defines", () => {
  for (const name of TONE_NAMES.filter((tone) => tone !== "neutral")) {
    const classes = TONES[name];
    const colour = classes.text.replace("text-", "");
    expect(CSS, name).toContain(`--color-${colour}:`);
    expect(classes.fill).toBe(`bg-${colour}`);
    expect(classes.badge).toContain(`text-${colour}`);
    expect(classes.solid).toContain(`bg-${colour}`);
    expect(classes.outline).toContain(`text-${colour}`);
    expect(classes.ghost).toContain(`text-${colour}`);
  }
});

it("an unknown tone from a newer portal is drawn neutral", () => {
  expect(toneOf("magenta")).toBe(TONES.neutral);
  expect(toneOf("indigo").text).toBe("text-palette-indigo");
});
