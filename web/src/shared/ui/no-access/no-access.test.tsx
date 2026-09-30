import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { TestIntl } from "@/shared/i18n";

import { NoAccess } from "./no-access";

it("says there is no access and links home", () => {
  render(
    <TestIntl>
      <NoAccess />
    </TestIntl>,
  );
  expect(screen.getByRole("status")).toHaveTextContent("You have no access to this page");
  expect(screen.getByRole("link", { name: "Back to home" })).toHaveAttribute("href", "/");
});
