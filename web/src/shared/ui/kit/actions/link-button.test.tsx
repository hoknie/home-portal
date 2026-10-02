import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { LinkButton } from "./link-button";

it("is a link that looks like the outline button", () => {
  render(<LinkButton href="/admin/services/">services</LinkButton>);
  const link = screen.getByRole("link", { name: "services" });
  expect(link.getAttribute("href")).toMatch(/^\/admin\/services\/?$/);
  expect(link.className).toContain("border-glass-edge");
});
