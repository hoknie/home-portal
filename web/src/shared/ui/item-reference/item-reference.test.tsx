import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { ItemReference } from "./item-reference";

it("names the kind and links the item, and a second item below it", () => {
  render(<ItemReference kind="Automation" text="NAS down" href="/admin/automations/edit/?id=nas-down" also={{ kind: "Workflow", text: "Revive", href: "/admin/workflows/revive/" }} />);
  expect(screen.getByText("Automation")).toBeInTheDocument();
  expect(screen.getByRole("link", { name: "NAS down" })).toHaveAttribute("href", expect.stringMatching(/^\/admin\/automations\/edit\/?\?id=nas-down$/));
  expect(screen.getByRole("link", { name: "Revive" })).toHaveAttribute("href", expect.stringMatching(/^\/admin\/workflows\/revive\/?$/));
});

it("shows an item without an address as plain text", () => {
  render(<ItemReference kind="Script" text="backup.sh" href={null} mono />);
  expect(screen.getByText("backup.sh")).toBeInTheDocument();
  expect(screen.queryByRole("link")).toBeNull();
});
