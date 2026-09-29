import { fireEvent, screen } from "@testing-library/react";
import { useState } from "react";
import { expect, it, vi } from "vitest";

import { renderWithProviders } from "@/shared/lib/testing";

import { CodeArea } from "./code-area";

function Editable({ initial, onSave }: { initial: string; onSave?: () => void }) {
  const [value, setValue] = useState(initial);
  return <CodeArea aria-label="Script" value={value} onChange={setValue} onSave={onSave} />;
}

it("numbers every line", () => {
  renderWithProviders(<Editable initial={"#!/bin/sh\necho hi\nexit 0"} />);
  expect(screen.getByText("3")).toBeInTheDocument();
});

it("tab inserts two spaces and keeps the focus in the field", () => {
  renderWithProviders(<Editable initial="ab" />);
  const field = screen.getByRole("textbox", {
    name: "Script",
  }) as HTMLTextAreaElement;
  field.focus();
  field.setSelectionRange(1, 1);
  fireEvent.keyDown(field, { key: "Tab" });
  expect(field).toHaveValue("a  b");
  expect(field).toHaveFocus();
});

it("control or command and s saves", () => {
  const onSave = vi.fn();
  renderWithProviders(<Editable initial="x" onSave={onSave} />);
  const field = screen.getByRole("textbox", { name: "Script" });
  fireEvent.keyDown(field, { key: "s", metaKey: true });
  fireEvent.keyDown(field, { key: "s", ctrlKey: true });
  expect(onSave).toHaveBeenCalledTimes(2);
});

it("paints the text under the field with its tokens", () => {
  renderWithProviders(<Editable initial={'#!/bin/sh\n# @arg service <text>\necho "$1"'} />);
  const painted = screen.getByTestId("code-highlight");
  expect(painted.querySelector('[data-token="tag"]')).toHaveTextContent("@arg");
  expect(painted.querySelector('[data-token="type"]')).toHaveTextContent("<text>");
  expect(painted.querySelector('[data-token="string"]')).toHaveTextContent('"$1"');
});
