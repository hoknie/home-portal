import { expect, it } from "vitest";

import { CUSTOM_TEMPLATES } from "./templates";

const text = (key: string) => key;

it("offers the ready custom widgets, one of them a row that holds a column with shared widths", () => {
  expect(CUSTOM_TEMPLATES.map((template) => template.key)).toEqual(["stat", "progress", "list", "table", "pairs", "buttons", "overview"]);
  const [row] = CUSTOM_TEMPLATES.find((template) => template.key === "overview")?.blocks(text) ?? [];
  const blocks = row.blocks as Record<string, unknown>[];
  expect(row).toMatchObject({ kind: "row", widths: [7, 5] });
  expect(blocks.map((block) => block.kind)).toEqual(["stat", "column"]);
  expect((row.widths as number[]).length).toBe(blocks.length);
});
