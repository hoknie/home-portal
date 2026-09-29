import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, expect, it } from "vitest";

import { AddressLink } from "./address-link";

beforeEach(() => {
  window.history.replaceState(null, "", "/admin/workflows/");
});

it("a plain click moves to its address without leaving the page", () => {
  render(<AddressLink href="/admin/workflows/revive/">Revive</AddressLink>);
  const link = screen.getByRole("link", { name: "Revive" });
  expect(link).toHaveAttribute("href", "/admin/workflows/revive/");
  const followed = fireEvent.click(link);
  expect(followed).toBe(false);
  expect(window.location.pathname).toBe("/admin/workflows/revive/");
});

it("a click with Ctrl or Cmd, or with another button, is left to the browser", () => {
  render(<AddressLink href="/admin/workflows/revive/">Revive</AddressLink>);
  const link = screen.getByRole("link", { name: "Revive" });
  expect(fireEvent.click(link, { ctrlKey: true })).toBe(true);
  expect(fireEvent.click(link, { metaKey: true })).toBe(true);
  expect(fireEvent.click(link, { button: 1 })).toBe(true);
  expect(window.location.pathname).toBe("/admin/workflows/");
});
