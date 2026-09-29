import { act, fireEvent, render, renderHook, screen } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { pushAddress } from "@/shared/lib/navigation";
import { AddressLink } from "@/shared/ui/address-link";

import { useLeaveGuard } from "./use-leave-guard";

beforeEach(() => {
  window.history.replaceState(null, "", "/admin/workflows/revive/edit/");
});

afterEach(() => {
  vi.restoreAllMocks();
});

it("a dirty page asks before an address move and stays when refused", () => {
  const confirm = vi.spyOn(window, "confirm").mockReturnValue(false);
  const { rerender } = renderHook(({ dirty }) => useLeaveGuard(dirty, "Leave?"), { initialProps: { dirty: true } });
  act(() => {
    pushAddress("/admin/workflows/revive/");
  });
  expect(confirm).toHaveBeenCalledWith("Leave?");
  expect(window.location.pathname).toBe("/admin/workflows/revive/edit/");
  rerender({ dirty: false });
  act(() => {
    pushAddress("/admin/workflows/revive/");
  });
  expect(confirm).toHaveBeenCalledTimes(1);
  expect(window.location.pathname).toBe("/admin/workflows/revive/");
});

it("an address link asks once, not twice", () => {
  const confirm = vi.spyOn(window, "confirm").mockReturnValue(true);
  function Page() {
    useLeaveGuard(true, "Leave?");
    return <AddressLink href="/admin/workflows/revive/">Back</AddressLink>;
  }
  render(<Page />);
  fireEvent.click(screen.getByRole("link", { name: "Back" }));
  expect(confirm).toHaveBeenCalledTimes(1);
  expect(window.location.pathname).toBe("/admin/workflows/revive/");
});
