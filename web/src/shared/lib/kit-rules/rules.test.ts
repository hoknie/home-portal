import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";

import { describe, expect, it } from "vitest";

import { governed, violations } from "./rules";

const ROOT = join(process.cwd(), "src");

function walk(directory: string): string[] {
  return readdirSync(directory).flatMap((name) => {
    const path = join(directory, name);
    return statSync(path).isDirectory() ? walk(path) : [path];
  });
}

function sourcesOutsideTheKit() {
  return walk(ROOT)
    .map((path) => relative(ROOT, path))
    .filter(governed)
    .map((path) => ({ path, text: readFileSync(join(ROOT, path), "utf8") }));
}

describe("screens take their elements and classes from the kit", () => {
  it("no file outside the kit breaks a kit rule, with no exceptions", () => {
    expect(sourcesOutsideTheKit().flatMap(violations)).toEqual([]);
  });

  it("lint exempts only the kit and the tests from the element rules", () => {
    const config = readFileSync(join(process.cwd(), "eslint.config.mjs"), "utf8");
    const ignores = /ignores: \[([^\]]*)\],\n\s*rules: \{\n\s*"no-restricted-syntax"/.exec(config);
    expect(ignores?.[1].split(",").map((entry) => entry.trim())).toEqual(['"src/shared/ui/kit/**"', '"src/**/*.test.ts"', '"src/**/*.test.tsx"']);
  });
});

describe("the kit rules themselves", () => {
  const widget = (text: string) => violations({ path: "widgets/x/ui/x.tsx", text });

  it("catch raw controls and headings, but not kit elements", () => {
    expect(widget("export const A = () => <button>x</button>;\nexport const B = () => <Button>x</Button>;\nexport const C = () => <h2>t</h2>;\n")).toEqual([
      "widgets/x/ui/x.tsx:1 renders a raw <button>",
      "widgets/x/ui/x.tsx:3 renders a raw <h2>",
    ]);
  });

  it("catch a Radix import", () => {
    expect(widget('import { Popover } from "radix-ui";\nimport { X } from "@radix-ui/react-popover";\n')).toEqual([
      "widgets/x/ui/x.tsx:1 imports radix-ui",
      "widgets/x/ui/x.tsx:2 imports @radix-ui/react-popover",
    ]);
  });

  it("catch sizes, colours, glass, shadows and tones off the scale, with variants", () => {
    expect(widget('export const a = "text-[11px] text-muted-foreground/70 hover:bg-white dark:text-sky-300 glass-panel shadow-sm text-3xl";\n')).toEqual([
      "widgets/x/ui/x.tsx:1 uses an arbitrary value: text-[11px]",
      "widgets/x/ui/x.tsx:1 uses a palette colour: bg-white",
      "widgets/x/ui/x.tsx:1 uses a palette colour: text-sky-300",
      "widgets/x/ui/x.tsx:1 uses a glass, surface, shadow or blur class: glass-panel",
      "widgets/x/ui/x.tsx:1 uses a glass, surface, shadow or blur class: shadow-sm",
      "widgets/x/ui/x.tsx:1 uses a text tone with opacity: text-muted-foreground/70",
      "widgets/x/ui/x.tsx:1 uses a text size off the scale: text-3xl",
    ]);
  });

  it("leave the scale, tokens and plain words alone", () => {
    expect(widget('export const a = "text-xs text-muted-foreground bg-glass-tint border-glass-edge rounded-xl ring-[3px] gap-4";\nexport const b = t("shadow.label");\n')).toEqual([]);
  });

  it("do not govern the kit, the tests or generated schemas", () => {
    expect(["shared/ui/kit/actions/button.tsx", "widgets/x/ui/x.test.tsx", "shared/api/generated/x.ts"].some(governed)).toBe(false);
    expect(governed("widgets/x/ui/x.tsx")).toBe(true);
  });
});
