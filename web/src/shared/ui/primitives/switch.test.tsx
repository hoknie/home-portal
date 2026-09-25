import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";

import { Switch } from "./switch";

it("toggles and reports the new state", async () => {
  const change = vi.fn();
  render(<Switch aria-label="probe" onCheckedChange={change} />);
  await userEvent.click(screen.getByRole("switch", { name: "probe" }));
  expect(change).toHaveBeenCalledWith(true);
});
