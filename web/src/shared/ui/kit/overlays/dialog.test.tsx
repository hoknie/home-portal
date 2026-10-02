import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it } from "vitest";

import { Dialog, DialogContent, DialogDescription, DialogTitle, DialogTrigger } from "./dialog";

it("opens and closes with a labelled close button", async () => {
  render(
    <Dialog>
      <DialogTrigger>open</DialogTrigger>
      <DialogContent closeLabel="close it">
        <DialogTitle>heading</DialogTitle>
        <DialogDescription>details</DialogDescription>
      </DialogContent>
    </Dialog>,
  );
  await userEvent.click(screen.getByText("open"));
  expect(screen.getByRole("dialog")).toHaveClass("glass-overlay");
  await userEvent.click(screen.getByRole("button", { name: "close it" }));
  expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
});

it("only dims the page behind it, so a single surface blurs", async () => {
  render(
    <Dialog>
      <DialogTrigger>open</DialogTrigger>
      <DialogContent closeLabel="close it">
        <DialogTitle>heading</DialogTitle>
        <DialogDescription>details</DialogDescription>
      </DialogContent>
    </Dialog>,
  );
  await userEvent.click(screen.getByText("open"));
  const overlay = document.querySelector("[data-slot='dialog-overlay']");
  expect(overlay).not.toBeNull();
  expect(overlay?.className).not.toMatch(/backdrop-blur/);
  expect(overlay?.className).toMatch(/bg-black/);
});
