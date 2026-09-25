import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";

import { TestIntl } from "@/shared/i18n";

import { ConfirmDialog } from "./confirm-dialog";

function renderDialog(onConfirm = vi.fn(), onOpenChange = vi.fn()) {
  render(
    <TestIntl>
      <ConfirmDialog open title="Delete NAS?" description="Gone for good" confirmLabel="Delete" onConfirm={onConfirm} onOpenChange={onOpenChange} />
    </TestIntl>,
  );
  return { onConfirm, onOpenChange };
}

it("confirms only when the destructive button is pressed", async () => {
  const { onConfirm } = renderDialog();
  await userEvent.click(screen.getByRole("button", { name: "Delete" }));
  expect(onConfirm).toHaveBeenCalledOnce();
});

it("cancels without confirming", async () => {
  const { onConfirm, onOpenChange } = renderDialog();
  await userEvent.click(screen.getByRole("button", { name: "Отмена" }));
  expect(onOpenChange).toHaveBeenCalledWith(false);
  expect(onConfirm).not.toHaveBeenCalled();
});
