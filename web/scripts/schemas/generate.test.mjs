import { readFileSync } from "node:fs";
import { join } from "node:path";

import { expect, it } from "vitest";
import { z } from "zod";

import { GENERATED, answers, generated, index } from "./files.mjs";
import { UnsupportedSchema, generate } from "./generate.mjs";

function load(code) {
  const names = [...code.matchAll(/export const (\w+)/g)].map((match) => match[1]);
  const body = code.replace('import { z } from "zod";', "").replace(/export type [^;]+;/g, "").replace(/export const /g, "const ");
  return new Function("z", `${body}\nreturn { ${names.join(", ")} };`)(z);
}

const SMALL = {
  title: "Garden",
  type: "object",
  properties: {
    name: { type: "string" },
    state: { $ref: "#/$defs/State" },
    owner: { anyOf: [{ $ref: "#/$defs/Owner" }, { type: "null" }] },
    beds: { type: "array", items: { type: "integer" } },
    labels: { type: "object", additionalProperties: { type: "string" } },
    shape: { oneOf: [{ type: "object", properties: { round: { type: "number" } }, required: ["round"] }, { type: "object", properties: { side: { type: "number" } }, required: ["side"] }] },
    note: { type: ["string", "null"] },
    extra: true,
  },
  required: ["name", "state", "owner", "shape"],
  $defs: {
    State: { type: "string", enum: ["green", "dry", "unknown"], "x-open": "unknown" },
    Owner: { type: "object", properties: { id: { type: "string" } }, required: ["id"] },
  },
};

it("objects, open enums, references, nullable, records and unions become one zod module", () => {
  const { gardenSchema, stateSchema } = load(generate(SMALL, "garden"));
  const parsed = gardenSchema.parse({ name: "back", state: "flooded", owner: null, shape: { side: 2 } });
  expect(parsed).toEqual({ name: "back", state: "unknown", owner: null, beds: [], shape: { side: 2 }, note: null });
  expect(stateSchema.parse("dry")).toBe("dry");
  expect(gardenSchema.parse({ name: "x", state: "dry", owner: { id: "me" }, labels: { a: "b" }, shape: { round: 1 } }).labels).toEqual({ a: "b" });
  expect(() => gardenSchema.parse({ name: 3, state: "dry", owner: null, shape: { side: 1 } })).toThrow();
});

it("an unsupported construct fails generation and names its path", () => {
  const tuple = { type: "object", properties: { pair: { type: "array", prefixItems: [{ type: "string" }] } } };
  expect(() => generate(tuple, "pairs")).toThrow(new UnsupportedSchema("pairs.pair", "an array without one item schema"));
  const loop = { type: "object", properties: { next: { $ref: "#/$defs/Node" } }, $defs: { Node: { type: "object", properties: { next: { $ref: "#/$defs/Node" } } } } };
  expect(() => generate(loop, "chain")).toThrow(/a recursive definition Node/);
});

it("every generated file matches what its schema generates now", () => {
  const names = answers();
  expect(names.length).toBeGreaterThan(0);
  for (const answer of names) {
    expect(readFileSync(join(GENERATED, `${answer}.ts`), "utf8"), answer).toBe(generated(answer));
  }
  expect(readFileSync(join(GENERATED, "index.ts"), "utf8")).toBe(index(names));
});
