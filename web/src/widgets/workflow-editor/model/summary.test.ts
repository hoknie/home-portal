import { expect, it } from "vitest";

import { summaryOf } from "./summary";

it("a set node shows an equals sign only when it has one value", () => {
  expect(summaryOf({ id: "a", kind: "set", variable: "svc", value: "nas" }, [])).toEqual({ text: "svc = nas" });
  expect(summaryOf({ id: "a", kind: "set", variable: "svc", json: "[1]" }, [])).toEqual({ text: "svc = [1]" });
  expect(summaryOf({ id: "a", kind: "set", variable: "svc", list: ["a", "b"] }, [])).toEqual({ key: "summaries.setList", params: { variable: "svc", count: 2 } });
  expect(summaryOf({ id: "a", kind: "set", variable: "svc", object: { a: "1", b: "2", c: "3" } }, [])).toEqual({ key: "summaries.setObject", params: { variable: "svc", count: 3 } });
  expect(summaryOf({ id: "a", kind: "set", variable: "svc", value: "" }, [])).toEqual({ text: "svc" });
});
