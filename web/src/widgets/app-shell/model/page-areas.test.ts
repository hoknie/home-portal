import { expect, it } from "vitest";

import { areaOfPage } from "./page-areas";

it("every admin page belongs to the area of its section, subpages included", () => {
  expect(areaOfPage("/admin/proxy/")).toBe("proxy");
  expect(areaOfPage("/admin/workflows/backup/history/3/")).toBe("workflows");
  expect(areaOfPage("/admin/permissions")).toBe("host-permissions");
  expect(areaOfPage("/admin/webhooks/edit/")).toBe("webhooks");
  expect(areaOfPage("/")).toBeNull();
});
