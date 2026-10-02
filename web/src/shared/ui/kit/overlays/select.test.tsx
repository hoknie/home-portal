import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeAll, expect, it, vi } from "vitest";

import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./select";

beforeAll(() => {
  Element.prototype.hasPointerCapture = () => false;
  Element.prototype.scrollIntoView = () => {};
});

it("shows the chosen value and reports another choice", async () => {
  const change = vi.fn();
  render(
    <Select value="all" onValueChange={change}>
      <SelectTrigger aria-label="source">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="all">everything</SelectItem>
        <SelectItem value="backup">backup</SelectItem>
      </SelectContent>
    </Select>,
  );
  expect(screen.getByRole("combobox", { name: "source" })).toHaveTextContent("everything");
  await userEvent.click(screen.getByRole("combobox", { name: "source" }));
  await userEvent.click(await screen.findByRole("option", { name: "backup" }));
  expect(change).toHaveBeenCalledWith("backup");
});
