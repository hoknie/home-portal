import { describe, expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { depthOf, emptyRow, incomplete, joinOf, joined, rowsOf, summary } from "./conditions";
import { workflowCatalogueSchema } from "./schema";

const catalogue = workflowCatalogueSchema.parse(apiSamples.workflowCatalogue);

describe("workflow conditions", () => {
  it("rows join as all or any and one plain row stays plain", () => {
    const row = { left: "{{steps.ping.status}}", op: "==", right: "200" };
    expect(joined("all", [row])).toEqual(row);
    const any = joined("any", [row, emptyRow()]);
    expect(joinOf(any)).toBe("any");
    expect(rowsOf(any)).toHaveLength(2);
    expect(rowsOf(row)).toEqual([row]);
  });

  it("a summary reads the condition and depth counts groups", () => {
    const nested = { all: [{ left: "a", op: "==", right: "1" }, { any: [{ left: "b", op: "is-empty" }, { left: "c", op: ">", right: "2" }] }] };
    expect(summary(nested)).toBe("a == 1 && (b is-empty || c > 2)");
    expect(depthOf(nested)).toBe(2);
    expect(depthOf({ left: "a", op: "==", right: "1" })).toBe(0);
  });

  it("a row is incomplete without a left value, or without a right value its operator needs", () => {
    expect(incomplete({ left: "", op: "==", right: "1" }, catalogue)).toBe(true);
    expect(incomplete({ left: "a", op: "is-empty" }, catalogue)).toBe(false);
    expect(incomplete({ left: "a", op: "==" }, catalogue)).toBe(true);
    expect(incomplete({ all: [] }, catalogue)).toBe(true);
  });
});
