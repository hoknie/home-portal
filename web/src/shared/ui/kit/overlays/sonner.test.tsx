import { act, render, screen } from "@testing-library/react";
import { toast } from "sonner";
import { expect, it } from "vitest";

import { Toaster } from "./sonner";

it("shows a neutral toast on an opaque surface, so stacked toasts never blur twice", async () => {
  const { container } = render(<Toaster />);
  act(() => {
    toast("saved");
  });
  const text = await screen.findByText("saved");
  expect(text.closest("[data-sonner-toast]")).not.toHaveClass("glass-overlay");
  const toaster = container.ownerDocument.querySelector<HTMLElement>("[data-sonner-toaster]");
  expect(toaster?.style.getPropertyValue("--normal-bg")).toBe("var(--glass-overlay-solid)");
});
