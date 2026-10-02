import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it } from "vitest";

import { Popover, PopoverContent, PopoverTrigger } from "./popover";

it("opens a raised surface beside its trigger", async () => {
  render(
    <Popover>
      <PopoverTrigger>more</PopoverTrigger>
      <PopoverContent>details</PopoverContent>
    </Popover>,
  );
  await userEvent.click(screen.getByText("more"));
  expect(screen.getByText("details")).toHaveClass("surface-raised");
});
