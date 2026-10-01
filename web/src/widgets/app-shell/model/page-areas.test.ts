import { expect, it } from "vitest";

import { areaOfPage, areasOfPage } from "./page-areas";

it("every admin page belongs to the area of its section, subpages included", () => {
  expect(areaOfPage("/admin/proxy/")).toBe("proxy");
  expect(areaOfPage("/admin/workflows/backup/history/3/")).toBe("workflows");
  expect(areaOfPage("/admin/permissions")).toBe("host-permissions");
  expect(areaOfPage("/admin/webhooks/edit/")).toBe("webhooks");
  expect(areaOfPage("/")).toBeNull();
});

it("the run journal opens for whoever may read automations or workflows", () => {
  expect(areasOfPage("/admin/runs/")).toEqual(["automations", "workflows"]);
  expect(areasOfPage("/")).toEqual([]);
});
