import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { AlignCenter, AlignLeft, AlignRight } from "lucide-react";
import { useState } from "react";
import { expect, it } from "vitest";

import { IconChoice } from "./icon-choice";

type Align = "start" | "center" | "end";

function Harness({ swatch = false }: { swatch?: boolean }) {
  const [value, setValue] = useState<Align>("start");
  return (
    <IconChoice
      label="Alignment"
      value={value}
      onChange={setValue}
      swatch={swatch}
      options={[
        { value: "start", label: "Left", icon: <AlignLeft /> },
        { value: "center", label: "Centre", icon: <AlignCenter /> },
        { value: "end", label: "Right", icon: <AlignRight /> },
      ]}
    />
  );
}

it("is a radio group of icons, each named by its label, with the chosen one checked", async () => {
  render(<Harness />);
  const group = screen.getByRole("radiogroup", { name: "Alignment" });
  expect(within(group).getAllByRole("radio").map((item) => item.getAttribute("aria-label"))).toEqual(["Left", "Centre", "Right"]);
  expect(screen.getByRole("radio", { name: "Left" })).toHaveAttribute("aria-checked", "true");
  expect(screen.getByRole("radio", { name: "Left" })).toHaveClass("aria-checked:border-primary");
  await userEvent.click(screen.getByRole("radio", { name: "Right" }));
  expect(screen.getByRole("radio", { name: "Right" })).toHaveAttribute("aria-checked", "true");
  expect(screen.getByRole("radio", { name: "Left" })).toHaveAttribute("aria-checked", "false");
});

it("moves the choice with the arrow keys and wraps around", async () => {
  render(<Harness />);
  await userEvent.click(screen.getByRole("radio", { name: "Left" }));
  const press = async (key: string, expected: string) => {
    await userEvent.keyboard(`{${key}>}`);
    await waitFor(() => expect(screen.getByRole("radio", { name: expected })).toHaveAttribute("aria-checked", "true"));
    expect(screen.getByRole("radio", { name: expected })).toHaveFocus();
    await userEvent.keyboard(`{/${key}}`);
  };
  await press("ArrowRight", "Centre");
  await press("ArrowLeft", "Left");
  await press("ArrowLeft", "Right");
});

it("shows the label in a tooltip on hover", async () => {
  render(<Harness />);
  await userEvent.hover(screen.getByRole("radio", { name: "Centre" }));
  expect(await screen.findByRole("tooltip")).toHaveTextContent("Centre");
});

it("draws round swatches in swatch mode", () => {
  render(<Harness swatch />);
  expect(screen.getByRole("radio", { name: "Left" })).toHaveClass("rounded-full");
});
