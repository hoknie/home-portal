import { screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";

import { renderWithProviders } from "@/shared/lib/testing";

import { Redirect } from "./redirect";

const replace = vi.fn();

vi.mock("next/navigation", () => ({ useRouter: () => ({ replace, push: vi.fn() }) }));

afterEach(() => replace.mockReset());

it("sends the browser to the new address and keeps the query string", () => {
  Object.defineProperty(window, "location", { value: { ...window.location, search: "?edit=media" }, writable: true });
  renderWithProviders(<Redirect to="/admin/services/" />);
  expect(replace).toHaveBeenCalledWith("/admin/services/?edit=media");
  expect(screen.getByText("Moving to the new address…")).toBeInTheDocument();
});
