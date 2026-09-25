import { act, render, screen } from "@testing-library/react";
import { toast } from "sonner";
import { expect, it } from "vitest";

import { Toaster } from "./sonner";

it("shows a neutral toast on glass", async () => {
  render(<Toaster />);
  act(() => {
    toast("saved");
  });
  const text = await screen.findByText("saved");
  expect(text.closest("[data-sonner-toast]")).toHaveClass("glass-overlay");
});
