import { describe, expect, it } from "vitest";

import { safeNext } from "./safe-next";

it("returns to a page of the portal and nowhere else", () => {
  expect(safeNext("/services/?id=nas")).toBe("/services/?id=nas");
  expect(safeNext(null)).toBe("/");
  expect(safeNext("https://evil.example")).toBe("/");
  expect(safeNext("//evil.example")).toBe("/");
  expect(safeNext("/login/")).toBe("/");
});

describe("a crafted return address", () => {
  it.each(["/\\evil.example", "/\t/evil.example", "/\n/evil.example", "//evil.example", "https://evil.example", "/%5Cevil.example/..%2F"])("%s leads home", (value) => {
    const path = safeNext(value);
    expect(new URL(path, "https://portal.example").origin).toBe("https://portal.example");
  });

  it("a backslash or a control character leads home", () => {
    expect(safeNext("/\\evil.example")).toBe("/");
    expect(safeNext("/\t/evil.example")).toBe("/");
  });

  it("a path on the portal keeps its query and fragment", () => {
    expect(safeNext("/admin/services/?id=nas#probe")).toBe("/admin/services/?id=nas#probe");
  });
});
