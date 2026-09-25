import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it } from "vitest";

import { Sheet, SheetContent, SheetDescription, SheetTitle, SheetTrigger } from "./sheet";

it("slides in and closes with a labelled close button", async () => {
  render(
    <Sheet>
      <SheetTrigger>open</SheetTrigger>
      <SheetContent closeLabel="close it">
        <SheetTitle>heading</SheetTitle>
        <SheetDescription>details</SheetDescription>
      </SheetContent>
    </Sheet>,
  );
  await userEvent.click(screen.getByText("open"));
  expect(screen.getByRole("dialog")).toHaveClass("glass-overlay");
  await userEvent.click(screen.getByRole("button", { name: "close it" }));
  expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
});
