import { fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { expect, it } from "vitest";

import { TagInput, optionsFor } from "./tag-input";

function Harness({ initial = [], max, onValues }: { initial?: string[]; max?: number; onValues?: (values: string[]) => void }) {
  const [values, setValues] = useState(initial);
  return (
    <TagInput
      id="groups"
      values={values}
      max={max}
      suggestions={["Media", "Network", "Cameras"]}
      onChange={(next) => {
        setValues(next);
        onValues?.(next);
      }}
      removeLabel={(value) => `remove ${value}`}
      createLabel={(value) => `create ${value}`}
    />
  );
}

function type(text: string) {
  const input = screen.getByRole("combobox");
  fireEvent.focus(input);
  fireEvent.change(input, { target: { value: text } });
  return input;
}

it("suggests the known values that contain what was typed, ignoring case", () => {
  render(<Harness />);
  type("me");
  expect(screen.getAllByRole("option").map((option) => option.textContent)).toEqual(["Media", "Cameras", "create me"]);
});

it("offers no creation when the text names a known value exactly", () => {
  expect(optionsFor("media", ["Media"], []).map((option) => option.value)).toEqual(["Media"]);
  expect(optionsFor("", ["Media", "Network"], ["Network"]).map((option) => option.value)).toEqual(["Media"]);
});

it("chooses a suggestion with the mouse and shows it as a chip", () => {
  const seen: string[][] = [];
  render(<Harness onValues={(values) => seen.push(values)} />);
  type("net");
  fireEvent.mouseDown(screen.getByRole("option", { name: "Network" }));
  expect(seen.at(-1)).toEqual(["Network"]);
  expect(screen.getByRole("button", { name: "remove Network" })).toBeInTheDocument();
});

it("adds free text on Enter", () => {
  const seen: string[][] = [];
  render(<Harness onValues={(values) => seen.push(values)} />);
  const input = type("Printers");
  fireEvent.keyDown(input, { key: "ArrowDown" });
  fireEvent.keyDown(input, { key: "Enter" });
  expect(seen.at(-1)).toEqual(["Printers"]);
});

it("moves through the options with the arrows and names the active one", () => {
  render(<Harness />);
  const input = type("");
  fireEvent.keyDown(input, { key: "ArrowDown" });
  const options = screen.getAllByRole("option");
  expect(input).toHaveAttribute("aria-activedescendant", options[1].id);
  expect(options[1]).toHaveAttribute("aria-selected", "true");
  fireEvent.keyDown(input, { key: "Enter" });
  expect(screen.getByRole("button", { name: "remove Network" })).toBeInTheDocument();
});

it("replaces the value when only one is allowed", () => {
  const seen: string[][] = [];
  render(<Harness initial={["Media"]} max={1} onValues={(values) => seen.push(values)} />);
  const input = type("Cameras");
  fireEvent.keyDown(input, { key: "Enter" });
  expect(seen.at(-1)).toEqual(["Cameras"]);
});

it("removes a chip by its button and the last one by Backspace", () => {
  const seen: string[][] = [];
  render(<Harness initial={["Media", "Network"]} onValues={(values) => seen.push(values)} />);
  fireEvent.click(screen.getByRole("button", { name: "remove Media" }));
  expect(seen.at(-1)).toEqual(["Network"]);
  fireEvent.keyDown(screen.getByRole("combobox"), { key: "Backspace" });
  expect(seen.at(-1)).toEqual([]);
});

it("closes the list on Escape", () => {
  render(<Harness />);
  const input = type("me");
  fireEvent.keyDown(input, { key: "Escape" });
  expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
});
