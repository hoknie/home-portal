import { expect, it } from "vitest";

import { CATEGORY_OF, MODULE_CATEGORIES, categoryParameter } from "./categories";
import { MODULE_NAMES } from "./schema";

it("every module has a category, and the categories come in their order", () => {
  expect(MODULE_NAMES.every((name) => MODULE_CATEGORIES.includes(CATEGORY_OF[name]))).toBe(true);
  expect(MODULE_CATEGORIES).toEqual(["network", "automation", "notifications", "access"]);
  expect(MODULE_NAMES.filter((name) => CATEGORY_OF[name] === "network")).toEqual(["proxy", "dns"]);
  expect(MODULE_NAMES.filter((name) => CATEGORY_OF[name] === "automation")).toEqual(["automations", "webhooks", "workflows"]);
});

it("an unknown or missing parameter means every category", () => {
  expect(categoryParameter("network")).toBe("network");
  expect(categoryParameter("unknown")).toBeNull();
  expect(categoryParameter(null)).toBeNull();
  expect(categoryParameter(undefined)).toBeNull();
});
