import { afterEach, expect, it } from "vitest";

import { clearChoice, readChoice, writeChoice } from "./choice";

afterEach(() => clearChoice());

it("remembers the chosen environment in a cookie the portal reads", () => {
  expect(readChoice()).toBeNull();
  writeChoice("internet");
  expect(document.cookie).toContain("portal_environment=internet");
  expect(readChoice()).toBe("internet");
});

it("forgets the choice when going back", () => {
  writeChoice("vpn");
  clearChoice();
  expect(readChoice()).toBeNull();
});
