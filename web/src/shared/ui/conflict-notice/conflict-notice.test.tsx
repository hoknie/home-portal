import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";

import { dictionaries } from "@/shared/i18n";
import { renderWithProviders } from "@/shared/lib/testing";

import { ConflictNotice } from "./conflict-notice";

const text = dictionaries.en.conflict;

it("reloads only after the person confirms that typed values are dropped", async () => {
  const onReload = vi.fn();
  renderWithProviders(<ConflictNotice onReload={onReload} onOverwrite={vi.fn()} />);
  await userEvent.click(screen.getByRole("button", { name: text.reload }));
  expect(onReload).not.toHaveBeenCalled();
  expect(screen.getByText(text.reloadDescription)).toBeInTheDocument();
  await userEvent.click(screen.getAllByRole("button", { name: text.reload }).at(-1)!);
  expect(onReload).toHaveBeenCalledOnce();
});

it("overwrites at once", async () => {
  const onOverwrite = vi.fn();
  renderWithProviders(<ConflictNotice onReload={vi.fn()} onOverwrite={onOverwrite} />);
  await userEvent.click(screen.getByRole("button", { name: text.overwrite }));
  expect(onOverwrite).toHaveBeenCalledOnce();
});
