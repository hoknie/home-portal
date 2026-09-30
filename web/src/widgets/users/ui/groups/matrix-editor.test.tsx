import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { expect, it } from "vitest";

import { type Rights, groupsSchema } from "@/entities/user";
import { apiSamples } from "@/shared/api";
import { TestIntl } from "@/shared/i18n";

import { MatrixEditor } from "./matrix-editor";

const matrix = groupsSchema.parse(apiSamples.groups).matrix;

function Editable({ onRights }: { onRights?: (rights: Rights) => void }) {
  const [rights, setRights] = useState<Rights>({});
  return (
    <TestIntl>
      <MatrixEditor
        matrix={matrix}
        rights={rights}
        onChange={(next) => {
          setRights(next);
          onRights?.(next);
        }}
      />
    </TestIntl>
  );
}

const box = (name: string) => screen.getByRole("checkbox", { name });

it("one action for every area ticks it only where the area offers it", async () => {
  let last: Rights = {};
  render(<Editable onRights={(rights) => (last = rights)} />);
  await userEvent.click(box("Every area: Read"));
  expect(box("Every area: Read")).toBeChecked();
  expect(last).toEqual(Object.fromEntries(matrix.filter((row) => row.actions.includes("read")).map((row) => [row.area, ["read"]])));
  expect(box("Services: Create")).not.toBeChecked();
  expect(box("Every area: Create")).toHaveAttribute("aria-checked", "false");
});

it("a whole area ticks its actions and leaves the columns mixed", async () => {
  render(<Editable />);
  await userEvent.click(box("Every action: Automations"));
  for (const action of ["Read", "Create", "Change", "Delete", "Run"]) {
    expect(box(`Automations: ${action}`)).toBeChecked();
    const column = box(`Every area: ${action}`) as HTMLInputElement;
    expect(column).toHaveAttribute("aria-checked", "mixed");
    expect(column.indeterminate).toBe(true);
  }
});

it("select all, clear one cell, then clear all", async () => {
  let last: Rights = {};
  render(<Editable onRights={(rights) => (last = rights)} />);
  const clear = screen.getByRole("button", { name: "Clear all" });
  expect(clear).toBeDisabled();
  await userEvent.click(screen.getByRole("button", { name: "Select all" }));
  expect(screen.getByRole("button", { name: "Select all" })).toBeDisabled();
  expect(screen.getAllByRole("checkbox").every((input) => (input as HTMLInputElement).checked)).toBe(true);
  await userEvent.click(box("Workflows: Run"));
  expect(box("Every action: Workflows")).toHaveAttribute("aria-checked", "mixed");
  expect(box("Every area: Run")).toHaveAttribute("aria-checked", "mixed");
  await userEvent.click(screen.getByRole("button", { name: "Clear all" }));
  expect(last).toEqual({});
  expect(screen.getAllByRole("checkbox").some((input) => (input as HTMLInputElement).checked)).toBe(false);
});

it("the read-only matrix has no bulk controls", () => {
  render(
    <TestIntl>
      <MatrixEditor matrix={matrix} rights={{ automations: ["read"] }} readOnly />
    </TestIntl>,
  );
  const table = screen.getByRole("table");
  expect(within(table).queryByRole("checkbox", { name: /^Every/ })).not.toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "Select all" })).not.toBeInTheDocument();
});
