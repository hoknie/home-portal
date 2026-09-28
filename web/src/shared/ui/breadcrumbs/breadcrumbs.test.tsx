import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it } from "vitest";

import { TestIntl } from "@/shared/i18n";

import { Breadcrumbs } from "./breadcrumbs";

const trail = [
  { label: "Home", href: "/" },
  { label: "Webhooks", href: "/admin/webhooks/" },
  { label: "Deploy", href: "/admin/webhooks/details/?id=d" },
  { label: "Edit" },
];

it("links every item but the last, which is the current page", () => {
  render(
    <TestIntl>
      <Breadcrumbs items={trail.slice(0, 2)} />
    </TestIntl>,
  );
  const nav = screen.getByRole("navigation", { name: "Breadcrumbs" });
  expect(within(nav).getByRole("link", { name: "Home" })).toHaveAttribute("href", "/");
  expect(within(nav).queryByRole("link", { name: "Webhooks" })).toBeNull();
  expect(within(nav).getByText("Webhooks")).toHaveAttribute("aria-current", "page");
  expect(within(nav).queryByRole("button")).toBeNull();
});

it("a long trail folds its middle on a narrow screen behind a button that shows it", async () => {
  render(
    <TestIntl>
      <Breadcrumbs items={trail} />
    </TestIntl>,
  );
  const webhooks = screen.getByRole("link", { name: "Webhooks" }).closest("li");
  expect(webhooks?.className).toContain("hidden sm:flex");
  expect(screen.getByRole("link", { name: "Deploy" }).closest("li")?.className).not.toContain("hidden");
  await userEvent.click(screen.getByRole("button", { name: "Show the whole path" }));
  expect(screen.getByRole("link", { name: "Webhooks" }).closest("li")?.className).not.toContain("hidden");
  expect(screen.queryByRole("button", { name: "Show the whole path" })).toBeNull();
});
