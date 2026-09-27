import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { TestIntl } from "@/shared/i18n";

import { ModuleOffNotice } from "./module-off-notice";

it("says the module is off and links to the modules page", () => {
  render(
    <TestIntl>
      <ModuleOffNotice name="DNS" />
    </TestIntl>,
  );
  expect(screen.getByRole("status")).toHaveTextContent("The DNS module is off");
  expect(screen.getByRole("link", { name: "Open modules" })).toHaveAttribute("href", "/admin/modules");
});
