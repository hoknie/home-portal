import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { expect, it, vi } from "vitest";

import { MultiSelect, type MultiSelectOption } from "./multi-select";

const OPTIONS: MultiSelectOption[] = [
  { value: "local", label: "local" },
  { value: "vpn", label: "vpn", disabled: true },
  { value: "internet", label: "internet" },
];

function Harness({ onChange }: { onChange: (values: string[]) => void }) {
  const [values, setValues] = useState(["internet"]);
  return (
    <MultiSelect
      label="Environments"
      options={OPTIONS}
      values={values}
      placeholder="None"
      onChange={(next) => {
        setValues(next);
        onChange(next);
      }}
    />
  );
}

it("shows the chosen values on one line and toggles them in the options' order, keeping the menu open", async () => {
  const onChange = vi.fn();
  render(<Harness onChange={onChange} />);
  const trigger = screen.getByRole("button", { name: "Environments" });
  expect(trigger).toHaveTextContent("internet");
  await userEvent.click(trigger);
  await userEvent.click(screen.getByRole("menuitemcheckbox", { name: "local" }));
  expect(onChange).toHaveBeenLastCalledWith(["local", "internet"]);
  expect(screen.getByRole("menuitemcheckbox", { name: "vpn" })).toHaveAttribute("aria-disabled", "true");
  await userEvent.click(screen.getByRole("menuitemcheckbox", { name: "internet" }));
  expect(onChange).toHaveBeenLastCalledWith(["local"]);
  await userEvent.keyboard("{Escape}");
  expect(screen.getByRole("button", { name: "Environments" })).toHaveTextContent("local");
});

it("shows the placeholder when nothing is chosen", () => {
  render(<MultiSelect label="Environments" options={OPTIONS} values={[]} placeholder="None" onChange={vi.fn()} />);
  expect(screen.getByRole("button", { name: "Environments" })).toHaveTextContent("None");
});
