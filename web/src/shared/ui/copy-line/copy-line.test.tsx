import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";

import { renderWithProviders } from "@/shared/lib/testing";

import { CopyLine } from "./copy-line";

it("copies its text to the clipboard", async () => {
  const user = userEvent.setup();
  const writeText = vi.spyOn(navigator.clipboard, "writeText");
  renderWithProviders(<CopyLine label="Address" text="https://portal/webhook/x" />);
  expect(screen.getByText("https://portal/webhook/x")).toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "Copy" }));
  await waitFor(() => expect(writeText).toHaveBeenCalledWith("https://portal/webhook/x"));
});
